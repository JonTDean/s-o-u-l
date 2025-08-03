//! 3-D grid back-ends (dense & sparse).
use glam::UVec3;
use crate::core::cell::CellState;

pub mod dense;
pub mod sparse;
pub mod modify;

use serde::{Deserialize, Serialize};

pub use dense ::DenseGrid;
pub use sparse::SparseGrid;
pub use modify::VoxelModify; 

/// Fallback edge-length when a sparse grid has no explicit size metadata.
const DEFAULT_SIDE: u32 = 256;

/// Run-time grid selection.
#[derive(Clone, Serialize, Deserialize)]
pub enum GridBackend {
    Dense (DenseGrid),
    Sparse(SparseGrid),
}

impl GridBackend {
    /// Grid **dimensions** as a `UVec3`.
    #[inline]
    pub fn dims(&self) -> UVec3 {
        match self {
            GridBackend::Dense(g)  => g.size,
            GridBackend::Sparse(_) => UVec3::new(DEFAULT_SIDE, DEFAULT_SIDE, 1),
        }
    }

    /// Write a single voxel.
    #[inline]
    pub fn write(&mut self, v: VoxelModify) {
        match self {
            GridBackend::Dense(g) => {
                if let Some(c) = g.get_mut(v.pos) {
                    c.state = CellState::from(v.value);
                }
            }
            GridBackend::Sparse(g) => g.set_state(v.pos, CellState::from(v.value)),
        }
    }
}