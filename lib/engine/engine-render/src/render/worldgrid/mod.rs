use glam::{IVec3, UVec3};
use bevy::prelude::*;
use simulation_kernel::{
    core::cell::CellState,
    atlas::{AtlasAllocator, GridSlice},
    grid::{DenseGrid, GridBackend},
};

pub mod minimap;

#[derive(Resource)]
pub struct WorldGrid {
    pub backend: GridBackend,
    alloc:       AtlasAllocator,
}

impl WorldGrid {
    pub fn new_dense(size: UVec3) -> Self {
        Self {
            backend: GridBackend::Dense(DenseGrid::blank(size)),
            alloc:   AtlasAllocator::new(size),
        }
    }

    /// Heap-friendly default for the new 3-D pipeline.
    pub fn new_sparse() -> Self {
        use simulation_kernel::grid::SparseGrid;
        Self {
            backend: GridBackend::Sparse(SparseGrid::default()),
            alloc:   AtlasAllocator::default(),        // 1024×1024 slice limit
        }
    }

    pub fn allocate(&mut self, size_xy: UVec2) -> Option<GridSlice> {
        self.alloc.allocate(UVec3::new(size_xy.x, size_xy.y, 1))
    }

    pub fn set(&mut self, slice: GridSlice, rel: IVec3, state: CellState) {
        let world = slice.offset + rel;
        match &mut self.backend {
            GridBackend::Dense(g)  => if let Some(c) = g.get_mut(world) { c.state = state; },
            GridBackend::Sparse(s) => s.set_state(world, state),
        }
    }
}
