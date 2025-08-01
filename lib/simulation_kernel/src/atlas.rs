//! Deterministic *single-layer 3-D* “guillotine” atlas allocator.
//
//! We keep the classic 2-D algorithm but embed it in voxel space
//! (`z == 0`).  Future work can extend the allocator to pack multiple
//! Z-layers (see SOUL-295).

use glam::{IVec3, UVec3};

/// Immutable slice metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSlice {
    pub offset: IVec3,   // centre-origin
    pub size:   UVec3,   // (w, h, depth=1)
}

#[derive(Debug, Clone)]
pub struct AtlasAllocator {
    size: UVec3,
    free: Vec<(IVec3, UVec3)>,
}

impl AtlasAllocator {
    #[inline]
    pub fn new(size: UVec3) -> Self {
        Self { size, free: vec![(IVec3::ZERO, size)] }
    }

    /// First-fit, depth = 1.
    #[inline]
    pub fn allocate(&mut self, want: UVec3) -> Option<GridSlice> {
        debug_assert_eq!(want.z, 1, "multi-layer slices NYI");

        let idx = self
            .free
            .iter()
            .position(|&(_, sz)| want.x <= sz.x && want.y <= sz.y)?;

        let (off, free_sz) = self.free.remove(idx);

        /* split right */
        if free_sz.x > want.x {
            self.free.push((
                IVec3::new(off.x + want.x as i32, off.y, 0),
                UVec3::new(free_sz.x - want.x, want.y, 1),
            ));
        }
        /* split below */
        if free_sz.y > want.y {
            self.free.push((
                IVec3::new(off.x, off.y + want.y as i32, 0),
                UVec3::new(free_sz.x, free_sz.y - want.y, 1),
            ));
        }

        /* re-centre */
        let centred = IVec3::new(
            off.x - self.size.x as i32 / 2,
            off.y - self.size.y as i32 / 2,
            0,
        );

        Some(GridSlice { offset: centred, size: want })
    }

    #[inline] pub fn free(&mut self, slice: GridSlice) { self.free.push((slice.offset, slice.size)); }
    #[inline] pub fn free_list(&self) -> &[(IVec3, UVec3)] { &self.free }
}

/* ------------------------------------------------------------- */
/* Default: 1 024 × 1 024 × 1 voxel atlas (single Z-layer)       */
/* ------------------------------------------------------------- */
impl Default for AtlasAllocator {
    fn default() -> Self {
        Self::new(UVec3::new(1_024, 1_024, 1))
    }
}