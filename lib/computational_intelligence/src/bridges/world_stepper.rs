//! computational_intelligence/bridges/world_stepper.rs
//! Parallel version:                                                     │
//! • `par_bridge()` → coarse‑grain parallelism across automata           │
//! • Rayon `par_iter_mut()` inside `step_dense_dyn_parallel()` for grids │

use bevy::prelude::*;
use serde_json::Value;
use rayon::prelude::*;
use engine_core::{
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::Dim2, Dim,
    },
    engine::grid::{DenseGrid, GridBackend, SparseGrid},
};

use crate::registry::AutomataRegistry;

/* ------------------------------------------------- */

const EMPTY: Value = Value::Null;

pub struct WorldStepperPlugin;
impl Plugin for WorldStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            step_every_automaton_parallel
                .in_set(engine_core::schedule::MainSet::Logic)
                .run_if(in_state(engine_core::state::AppState::InGame)),
        );
    }
}

/* ------------------------------------------------- */

fn step_every_automaton_parallel(mut reg: ResMut<AutomataRegistry>) {
    // 1 ── coarse‑grain: each automaton on its own Rayon task
    reg.iter_mut()
       .par_bridge()                      // converts iterator → ParallelIterator
       .for_each(|auto| {
           match &mut auto.grid {
               GridBackend::Dense(g)  => step_dense_dyn_parallel(g, &*auto.rule, &EMPTY),
               GridBackend::Sparse(s) => step_sparse_dyn_parallel(s, &*auto.rule, &EMPTY),
           }
       });
}

/* ------------------------------------------------- */
/* Fine‑grain helpers (dynamic dispatch + Rayon)     */
/* ------------------------------------------------- */

#[inline]
fn step_dense_dyn_parallel(
    grid:   &mut DenseGrid,
    rule:   &(dyn engine_core::core::AutomatonRule<D = Dim2> + Sync),
    params: &Value,
) {
    use bevy::math::IVec2;

    let snapshot = grid.cells.clone();          // read‑only copy
    let size     = grid.size;

    // Iterate *mutably* in parallel; each item is an exclusive &mut Cell
    grid.cells
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, cell)| {
            let x = (idx as u32 % size.x) as i32;
            let y = (idx as u32 / size.x) as i32;
            let p = IVec2::new(x, y);

            /* build Moore‑8 neighbourhood from the read‑only snapshot */
            let mut nbhd = [CellState::Dead; 8];
            for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
                let q = p + *off;
                if (0..size.x as i32).contains(&q.x) &&
                   (0..size.y as i32).contains(&q.y)
                {
                    nbhd[i] = snapshot[(q.y as u32 * size.x + q.x as u32) as usize].state;
                }
            }

            let ctx = CellCtx::<Dim2> {
                self_coord:    p,
                self_state:    cell.state,
                neighbourhood: &nbhd,
                memory:        &cell.memory,
                _marker:       std::marker::PhantomData,
            };

            if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
                cell.state  = state;
                cell.memory = memory;
            }
        });
}

#[inline]
fn step_sparse_dyn_parallel(
    grid:   &mut SparseGrid,
    rule:   &(dyn engine_core::core::AutomatonRule<D = Dim2> + Sync),
    params: &Value,
) {
    use bevy::math::IVec2;
    use std::collections::HashMap;
    use engine_core::core::cell::Cell;

    // clone() is cheap for sparse maps (pointer copies)
    let snapshot: HashMap<IVec2, Cell> = grid.map.clone();

    // Rayon can’t mutate the same HashMap in place safely, so we build a new one
    let new_map: HashMap<IVec2, Cell> = snapshot
        .par_iter()
        .map(|(&p, cell)| {
            let mut nbhd = [CellState::Dead; 8];
            for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
                nbhd[i] = snapshot.get(&(p + *off)).map_or(CellState::Dead, |c| c.state);
            }

            let ctx = CellCtx::<Dim2> {
                self_coord:    p,
                self_state:    cell.state,
                neighbourhood: &nbhd,
                memory:        &cell.memory,
                _marker:       std::marker::PhantomData,
            };

            match rule.next_state(ctx, params) {
                CellOutcome::Next { state, memory } => {
                    let mut updated = cell.clone();
                    updated.state  = state;
                    updated.memory = memory;
                    (p, updated)
                }
                CellOutcome::Unchanged => (p, cell.clone()),
            }
        })
        .collect();

    grid.map = new_map;
}
