//! Rayon parallel dense voxel stepper.

use glam::IVec3;
use rayon::prelude::*;
use serde_json::Value;

use crate::{
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::DenseGrid,
    AutomatonRule,
};

pub fn step_dense_dyn_parallel(
    grid:   &mut DenseGrid,
    rule:   &(dyn AutomatonRule<D = Dim> + Sync),
    params: &Value,
) {
    let snapshot = grid.voxels.clone();   // read-only
    let size     = grid.size;

    grid.voxels
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, voxel)| {
            let x =  idx as u32              % size.x;
            let y = (idx as u32 / size.x)    % size.y;
            let z =  idx as u32 / (size.x * size.y);
            let p = IVec3::new(x as i32, y as i32, z as i32);

            /* gather Moore-26 neighbourhood */
            let mut nbhd = [CellState::Dead; 26];
            for (i, off) in Dim::NEIGHBOUR_OFFSETS.iter().enumerate() {
                let q = p + *off;
                if (0..size.x as i32).contains(&q.x)
                    && (0..size.y as i32).contains(&q.y)
                    && (0..size.z as i32).contains(&q.z)
                {
                    let j = ((q.z as u32 * size.y + q.y as u32) * size.x + q.x as u32) as usize;
                    nbhd[i] = snapshot[j].state;
                }
            }

            let ctx = CellCtx::<Dim> {
                self_coord:    p,
                self_state:    voxel.state,
                neighbourhood: &nbhd,
                memory:        &voxel.memory,
                _marker:       std::marker::PhantomData,
            };

            if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
                voxel.state  = state;
                voxel.memory = memory;
            }
        });
}
