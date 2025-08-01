//! Single-threaded dense voxel stepper.

pub mod parallel;

use glam::IVec3;
use serde_json::Value;

use crate::{
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::DenseGrid,
    AutomatonRule,
};

pub fn step_dense<R: AutomatonRule<D = Dim>>(
    grid:   &mut DenseGrid,
    rule:   &R,
    params: &Value,
) {
    let snapshot = grid.voxels.clone();
    let mut next = snapshot.clone();

    for z in 0..grid.size.z as i32 {
        for y in 0..grid.size.y as i32 {
            for x in 0..grid.size.x as i32 {
                let p   = IVec3::new(x, y, z);
                let idx = grid.idx(p);

                /* gather Moore-26 neighbourhood */
                let mut nbhd = [CellState::Dead; 26];
                for (i, off) in Dim::NEIGHBOUR_OFFSETS.iter().enumerate() {
                    let q = p + *off;
                    if (0..grid.size.x as i32).contains(&q.x)
                        && (0..grid.size.y as i32).contains(&q.y)
                        && (0..grid.size.z as i32).contains(&q.z)
                    {
                        nbhd[i] = snapshot[grid.idx(q)].state;
                    }
                }

                let ctx = CellCtx::<Dim> {
                    self_coord: p,
                    self_state: snapshot[idx].state,
                    neighbourhood: &nbhd,
                    memory: &snapshot[idx].memory,
                    _marker: std::marker::PhantomData,
                };

                if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
                    next[idx].state  = state;
                    next[idx].memory = memory;
                }
            }
        }
    }
    grid.voxels = next;
}
