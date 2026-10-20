use glam::{IVec3, UVec3};
use serde_json::Value;
use simulation_kernel::{
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim3, Dimensionality},
    },
    grid::{DenseGrid, SparseGrid},
    stepper::AutomatonRule,
};
pub struct Life;
impl AutomatonRule for Life {
    type D = Dim3;
    fn next_state(&self, c: CellCtx<Dim3>, _: &Value) -> CellOutcome {
        let n = Dim3::NEIGHBOUR_OFFSETS
            .iter()
            .zip(c.neighbourhood)
            .filter(|(p, s)| p.z == 0 && **s != CellState::Dead)
            .count();
        CellOutcome::Next {
            state: if n == 3 || (n == 2 && c.self_state != CellState::Dead) {
                CellState::Alive(255)
            } else {
                CellState::Dead
            },
            memory: c.memory.clone(),
        }
    }
}
pub fn grids(side: u32, percent: u64) -> (DenseGrid, SparseGrid) {
    let mut dense = DenseGrid::blank(UVec3::new(side, side, 1));
    let mut sparse = SparseGrid::default();
    let mut seed = 0x534f554c_u64;
    for y in 0..side {
        for x in 0..side {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if (seed >> 32) % 100 < percent {
                let p = IVec3::new(x as i32, y as i32, 0);
                dense.set_state(p, CellState::Alive(255));
                sparse.set_state(p, CellState::Alive(255));
            }
        }
    }
    (dense, sparse)
}
