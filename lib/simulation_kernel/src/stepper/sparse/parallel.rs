//! Rayon parallel sparse voxel stepper.

use glam::IVec3;
use rayon::prelude::*;
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

pub fn step_sparse_dyn_parallel(
    grid:   &mut SparseGrid,
    rule:   &(dyn AutomatonRule<D = Dim> + Sync),
    params: &Value,
) {
    let snapshot: HashMap<IVec3, Cell> = grid.map.clone();
    let updates: Vec<(IVec3, CellState, serde_json::Value)> = snapshot
        .par_iter()
        .filter_map(|(&p, cell)| {
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
                CellOutcome::Next { state, memory } => Some((p, state, memory)),
                _ => None,
            }
        })
        .collect();

    for (p, state, memory) in updates {
        grid.map.entry(p).or_default().state  = state;
        grid.map.entry(p).or_default().memory = memory;
    }
}
