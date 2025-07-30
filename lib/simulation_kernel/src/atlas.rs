//! Deterministic 2-D “guillotine” atlas allocator
//! (no Bevy or RNG – the caller picks the free-slot selection strategy).

use glam::{IVec2, UVec2};

/// Immutable slice metadata (same as before, just lives in the kernel).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSlice {
    pub offset: IVec2,
    pub size:   UVec2,
}

/// First-fit guillotine allocator for a fixed-size 2-D atlas.
#[derive(Debug, Clone)]
pub struct AtlasAllocator {
    free: Vec<(IVec2, UVec2)>,
}

impl AtlasAllocator {
    /// Start with one big free rectangle (`size.x × size.y`).
    pub fn new(size: UVec2) -> Self {
        Self { free: vec![(IVec2::ZERO, size)] }
    }

    /// Deterministic first-fit allocation.
    pub fn allocate(&mut self, want: UVec2) -> Option<GridSlice> {
        let idx = self.free.iter().position(|(_, sz)| {
            want.x <= sz.x && want.y <= sz.y
        })?;

        let (off, free_sz) = self.free.remove(idx);

        /* guillotine split – right + below */
        if free_sz.x > want.x {
            self.free.push((IVec2::new(off.x + want.x as i32, off.y),
                            UVec2::new(free_sz.x - want.x, want.y)));
        }
        if free_sz.y > want.y {
            self.free.push((IVec2::new(off.x, off.y + want.y as i32),
                            UVec2::new(free_sz.x, free_sz.y - want.y)));
        }

        Some(GridSlice { offset: off, size: want })
    }

    /// Return a previously allocated slice (optional – not used yet).
    pub fn free(&mut self, slice: GridSlice) {
        self.free.push((slice.offset, slice.size));
        // NOTE: merging of adjacent rects is a future enhancement.
    }

    /// Exposed for tests.
    pub fn free_list(&self) -> &[(IVec2, UVec2)] { &self.free }
}
