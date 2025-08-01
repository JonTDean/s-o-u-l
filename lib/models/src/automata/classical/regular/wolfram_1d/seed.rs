//! Seeding helpers for the 1-D Wolfram elementary CA rules (z = 0 plane).

use glam::IVec3;
use engine_core::world::voxel_world::VoxelWorld;
use simulation_kernel::{core::cell::CellState, grid::GridBackend};

fn seed_middle_band_backend(grid: &mut GridBackend) {
    match grid {
        GridBackend::Dense(g) => {
            let y = g.size.y / 2;
            for x in 0..g.size.x {
                let idx = ((0 * g.size.y + y) * g.size.x + x) as usize; // z = 0
                g.voxels[idx].state = CellState::Alive(255);
            }
        }
        GridBackend::Sparse(s) => {
            for x in -32..=32 {
                s.set_state(IVec3::new(x, 0, 0), CellState::Alive(255));
            }
        }
    }
}

pub fn seed_rule30(grid: &mut GridBackend)  { seed_middle_band_backend(grid); }
pub fn seed_rule110(grid: &mut GridBackend) { seed_middle_band_backend(grid); }

pub fn seed_rule30_world(world: &mut VoxelWorld)  { seed_rule30(&mut world.backend); }
pub fn seed_rule110_world(world: &mut VoxelWorld) { seed_rule110(&mut world.backend); }
