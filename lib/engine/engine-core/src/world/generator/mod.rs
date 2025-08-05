//! Voxel-world procedural generation utilities.
//!
//! These helpers live in their own module so that both run-time systems
//! (e.g. *New Scenario* scene, autosave loader) and offline tooling can
//! share the exact same deterministic logic without copy-pasting code.

use bevy::prelude::*;
use glam::{IVec3, UVec3};
use simulation_kernel::grid::{dense::DenseGrid, GridBackend};

use super::voxel_world::VoxelWorld;

/* ─────────────────────────────────────────────────────────────── Options */

/// User-level options controlling how an empty [`VoxelWorld`] is created.
#[derive(Clone)]
pub struct VoxelWorldOptions {
    /// Dimensions of the grid in *voxels* (x, y, z).
    pub dimensions: IVec3,
    /// World-space size of *one* voxel edge.
    pub voxel_size: f32,
    /// Background colour used when sampling outside active cells.
    pub background: Color,
}

impl Default for VoxelWorldOptions {
    fn default() -> Self {
        Self {
            // Power-of-two cube keeps GPU work-group maths trivial.
            dimensions: IVec3::new(256, 256, 256),
            voxel_size: 1.0,
            background: Color::srgb(0.07, 0.07, 0.07),
        }
    }
}

/* ─────────────────────────────────────────────────────────────── Builder */

/// Build an empty [`VoxelWorld`] initialised to `CellState::Dead`.
///
/// The routine performs **no allocation on the calling thread** aside from
/// the grid itself and touches no ECS resources, making it freely usable
/// inside Rayon tasks or Bevy’s async compute systems.
pub fn create_voxel_world(opts: VoxelWorldOptions) -> VoxelWorld {
    // 1 ░ Dense grid with every cell set to `Dead`.
    let size = UVec3::new(
        opts.dimensions.x as u32,
        opts.dimensions.y as u32,
        opts.dimensions.z as u32,
    );
    let dense_grid = DenseGrid::blank(size);

    // 2 ░ Wrap in the run-time backend enum.
    let backend = GridBackend::Dense(dense_grid);

    // 3 ░ Return the Bevy resource.
    VoxelWorld {
        backend,
        voxel_size: opts.voxel_size,
        bg_color: opts.background,
    }
}
