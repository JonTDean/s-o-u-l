//! Bevy resource that owns a 3-D voxel grid (dense **or** sparse).

use bevy::prelude::*;
use glam::IVec3;
use simulation_kernel::{
    core::{
        cell::CellState,
        dim::{Dim, Dimensionality},
    },
    grid::GridBackend,
};

/// Bevy resource that owns a 3‑D voxel grid (dense **or** sparse).
#[derive(Resource)]
pub struct VoxelWorld {
    /// Dense or sparse grid backend storing cell state.
    pub backend: GridBackend,
    /// World‑space size of one voxel edge.
    pub voxel_size: f32,
    /// Background colour used when rendering empty cells.
    pub bg_color: Color,
}

impl VoxelWorld {
    /// Returns the 26-neighbour Moore neighbourhood around `coord`.
    #[inline]
    pub fn neighbourhood(&self, coord: IVec3) -> [CellState; 26] {
        let mut n = [CellState::Dead; 26];
        for (i, off) in Dim::NEIGHBOUR_OFFSETS.iter().enumerate() {
            let p = coord + *off;
            n[i] = match &self.backend {
                GridBackend::Dense(g) => g.get(p).map_or(CellState::Dead, |c| c.state),
                GridBackend::Sparse(g) => g.get(p).map_or(CellState::Dead, |c| c.state),
            };
        }
        n
    }
}
