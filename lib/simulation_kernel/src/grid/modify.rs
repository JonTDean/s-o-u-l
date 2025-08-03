// lib/simulation_kernel/src/grid/modify.rs
//! Single-voxel write helper used by debug utilities.
use glam::IVec3;

#[derive(Clone, Copy)]
pub struct VoxelModify {
    pub pos:   IVec3,
    pub value: u8,
}

impl VoxelModify {
    #[inline] pub fn new<P: Into<IVec3>>(pos: P, value: u8) -> Self {
        Self { pos: pos.into(), value }
    }
}