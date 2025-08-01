//! Single-threaded sparse voxel stepper.

pub mod parallel;

use glam::IVec3;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    core::{
        cell::{Cell, CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::SparseGrid,
    AutomatonRule,
};

pub fn step_sparse<R: AutomatonRule<D = Dim>>(
    grid:   &mut SparseGrid,
    rule:   &R,
    params: &Value,
) {
    let snapshot: HashMap<IVec3, Cell> = grid.map.clone();
    let mut next: HashMap<IVec3, Cell> = snapshot.clone();

    for (&p, cell) in &snapshot {
        let mut nbhd = [CellState::Dead; 26];
        for (i, off) in Dim::NEIGHBOUR_OFFSETS.iter().enumerate() {
            nbhd[i] = snapshot.get(&(p + *off)).map_or(CellState::Dead, |c| c.state);
        }

        let ctx = CellCtx::<Dim> {
            self_coord: p,
            self_state: cell.state,
            neighbourhood: &nbhd,
            memory: &cell.memory,
            _marker: std::marker::PhantomData,
        };

        match rule.next_state(ctx, params) {
            CellOutcome::Next { state, memory } => {
                next.entry(p).or_default().state  = state;
                next.entry(p).or_default().memory = memory;
            }
            CellOutcome::Unchanged => {}
        }
    }
    grid.map = next;
}
