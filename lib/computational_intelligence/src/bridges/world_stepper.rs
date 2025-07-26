//! Single‑threaded stepper that advances **all** registered automata.

use bevy::prelude::*;
use serde_json::Value;
use engine_core::{
    core::{
        CellCtx, 
        CellOutcome,
        cell::CellState,
        dim::Dim2, Dim,
    },
    engine::grid::{DenseGrid, GridBackend, SparseGrid},
};

use crate::registry::AutomataRegistry;

/* --------------------------------------------------------------------- */

const EMPTY: Value = Value::Null;            // default rule parameters

pub struct WorldStepperPlugin;
impl Plugin for WorldStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            step_every_automaton
                .in_set(engine_core::schedule::MainSet::Logic)
                .run_if(in_state(engine_core::state::AppState::InGame)),
        );
    }
}

/* --------------------------------------------------------------------- */

fn step_every_automaton(mut reg: ResMut<AutomataRegistry>) {
    for auto in reg.iter_mut() {
        match &mut auto.grid {
            GridBackend::Dense(g)  => step_dense_dyn(g, &*auto.rule, &EMPTY),
            GridBackend::Sparse(s) => step_sparse_dyn(s, &*auto.rule, &EMPTY),
        }
    }
}

/* --------------------------------------------------------------------- */
/* Dynamic helpers (trait‑object friendly)                               */
/* --------------------------------------------------------------------- */

#[inline]
fn step_dense_dyn(
    grid:   &mut DenseGrid,
    rule:   &dyn engine_core::core::AutomatonRule<D = Dim2>,
    params: &Value,
) {
    use bevy::math::IVec2;

    let snapshot = grid.cells.clone();              // read‑only copy
    let mut next = snapshot.clone();                // writable copy

    for y in 0..grid.size.y as i32 {
        for x in 0..grid.size.x as i32 {
            let p   = IVec2::new(x, y);
            let idx = grid.idx(p);

            /* build Moore‑8 neighbourhood */
            let mut nbhd = [CellState::Dead; 8];
            for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
                let q = p + *off;
                if (0..grid.size.x as i32).contains(&q.x)
                    && (0..grid.size.y as i32).contains(&q.y)
                {
                    nbhd[i] = snapshot[grid.idx(q)].state;
                }
            }

            let ctx = CellCtx::<Dim2> {
                self_coord:    p,
                self_state:    snapshot[idx].state,
                neighbourhood: &nbhd,
                memory:        &snapshot[idx].memory,
                _marker:       std::marker::PhantomData,
            };

            if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
                next[idx].state  = state;
                next[idx].memory = memory;
            }
        }
    }

    grid.cells = next;
}

#[inline]
fn step_sparse_dyn(
    grid:   &mut SparseGrid,
    rule:   &dyn engine_core::core::AutomatonRule<D = Dim2>,
    params: &Value,
) {
    use bevy::math::IVec2;
    use std::collections::HashMap;
    use engine_core::core::cell::Cell;

    let snapshot: HashMap<IVec2, Cell> = grid.map.clone();
    let mut next = snapshot.clone();

    for (&p, cell) in &snapshot {
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

        if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
            next.entry(p).or_default().state  = state;
            next.entry(p).or_default().memory = memory;
        }
    }

    grid.map = next;
}
