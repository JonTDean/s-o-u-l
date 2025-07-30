// lib/engine/src/renderer/worldgrid.rs
use glam::{IVec2, UVec2};
use bevy::prelude::*;
use simulation_kernel::{
    core::cell::CellState, 
    atlas::{
        AtlasAllocator, 
        GridSlice
    }, 
    grid::{
        DenseGrid, 
        GridBackend
    }
};

#[derive(Resource)]
pub struct WorldGrid {
    pub backend: GridBackend,
    alloc: AtlasAllocator,
}

impl WorldGrid {
    pub fn new_dense(size: UVec2) -> Self {
        Self {
            backend: GridBackend::Dense(DenseGrid::blank(size)),
            alloc:   AtlasAllocator::new(size),
        }
    }

    pub fn allocate(&mut self, size: UVec2) -> Option<GridSlice> {
        self.alloc.allocate(size)
    }

    /// Convenience – write a cell state through a slice relative coordinate.
    pub fn set(&mut self, slice: GridSlice, rel: IVec2, state: CellState) {
        let world = slice.offset + rel;
        match &mut self.backend {
            GridBackend::Dense(g)  => if let Some(c) = g.get_mut(world) { c.state = state; },
            GridBackend::Sparse(s) => s.set_state(world, state),
        }
    }
}
