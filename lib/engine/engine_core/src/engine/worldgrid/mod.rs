use bevy::prelude::*;
use crate::engine::grid::{DenseGrid, SparseGrid, GridBackend};

/// Immutable metadata every automaton stores instead of a full grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSlice {
    pub offset: IVec2,        // top‑left corner in world‑grid coordinates
    pub size:   UVec2,        // width × height
}

/// The single source of truth for spatial state.
#[derive(Resource)]
pub struct WorldGrid {
    pub backend: GridBackend,                 // dense or sparse physical data
    /// Free rectangle list → trivial “guillotine” allocator.
    free: Vec<(IVec2, UVec2)>,
}

impl WorldGrid {
    /// Create a blank world grid of the chosen backend & size.
    pub fn new_dense(size: UVec2) -> Self {
        Self {
            backend: GridBackend::Dense(DenseGrid::blank(size)),
            free: vec![(IVec2::ZERO, size)],
        }
    }
    pub fn new_sparse() -> Self {
        Self { backend: GridBackend::Sparse(SparseGrid::default()), free: vec![] }
    }

    /// Allocate a rectangular slice; now chooses **any** suitable rectangle
    /// at random instead of always the first one (reduces clustering).
    pub fn allocate(&mut self, size: UVec2) -> Option<GridSlice> {
        // ── ❶ collect every free slot that fits ──────────────────────────
        let mut rng   = rand::rng();
        let idx_opt   = rand::seq::IteratorRandom::choose(self.free
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                let free_sz = &entry.1; // entry = (&IVec2, &UVec2)
                size.x <= free_sz.x && size.y <= free_sz.y
            })
            .map(|(i, _)| i), &mut rng);          // <- 1 random index, if any

        let idx = idx_opt?;             // none ⇒ allocation fails
        let (off, free_sz) = self.free.remove(idx);
        // ── ❷ classic “guillotine” split of the remaining free area ──────
        if free_sz.x > size.x {
            self.free.push(( IVec2::new(off.x + size.x as i32, off.y),
                             UVec2::new(free_sz.x - size.x, size.y)));
        }
        if free_sz.y > size.y {
            self.free.push(( IVec2::new(off.x, off.y + size.y as i32),
                             UVec2::new(free_sz.x,       free_sz.y - size.y)));
        }
        Some(GridSlice { offset: off, size })
    }

    /// Convenience – write a cell state through a slice relative coordinate.
    pub fn set(&mut self, slice: GridSlice, rel: IVec2, state: crate::core::cell::CellState) {
        let world = slice.offset + rel;
        match &mut self.backend {
            GridBackend::Dense(g)  => if let Some(c) = g.get_mut(world) { c.state = state; },
            GridBackend::Sparse(s) => s.set_state(world, state),
        }
    }

    // Further helpers: neighbourhood(), iter_live(), etc.
}
