use bevy::math::IVec3;
use simulation_kernel::grid::{GridBackend, VoxelModify};

use crate::debugging::floor::SOLID_VOXEL;

/// Draw a **1-voxel-wide grid** every `STEP` cells.
pub(super) fn seed_checkerboard(grid: &mut GridBackend) {
    const STEP: u32 = 8;              // world-space grid spacing

    let sz = grid.dims();
    for y in 0..sz.y {
        for x in 0..sz.x {
            if x % STEP == 0 || y % STEP == 0 {
                let pos = IVec3::new(x as i32, y as i32, 0);
                grid.write(VoxelModify::new(pos, SOLID_VOXEL));
            }
        }                  
    }
}
