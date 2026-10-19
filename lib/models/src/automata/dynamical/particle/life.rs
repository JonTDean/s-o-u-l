//! Conway B3/S23 on independent XY planes with dead boundaries.
use glam::IVec3;
use serde_json::Value;
use simulation_kernel::{
    AutomatonRule,
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::{GridBackend, VoxelModify},
};

/// Binary Life; any positive density is a living cell.
pub struct Life;
impl AutomatonRule for Life {
    type D = Dim;
    fn next_state(&self, ctx: CellCtx<Dim>, _: &Value) -> CellOutcome {
        let count = Dim::NEIGHBOUR_OFFSETS
            .iter()
            .zip(ctx.neighbourhood)
            .filter(|(offset, state)| offset.z == 0 && matches!(state, CellState::Alive(1..)))
            .count();
        let alive = count == 3 || (count == 2 && matches!(ctx.self_state, CellState::Alive(1..)));
        let state = if alive {
            CellState::Alive(255)
        } else {
            CellState::Dead
        };
        CellOutcome::Next {
            state,
            memory: ctx.memory.clone(),
        }
    }
}

/// A glider centered in the finite workbench grid.
pub fn seed_life(grid: &mut GridBackend) {
    let centre = match grid {
        GridBackend::Dense(g) => (g.size / 2).as_ivec3(),
        GridBackend::Sparse(_) => IVec3::ZERO,
    };
    for (x, y) in [(0, -1), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        grid.write(VoxelModify {
            pos: centre + IVec3::new(x, y, 0),
            value: 255,
        });
    }
}
