//! Tiny, single-layer **2-D guillotine** atlas used to place rectangles
//! inside each layer of the 3-D voxel atlas.
//! (Extending to multiple layers would require a proper 3-D allocator.)
//
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

use super::textures::{MAX_H, MAX_W};
use bevy::prelude::*;

#[derive(Clone, Copy)]
struct Rect {
    off: UVec2,
    size: UVec2,
}

/// Minimal 2‑D guillotine allocator used per atlas layer.
#[derive(Resource)]
pub struct AtlasAllocator {
    free: Vec<Rect>,
}

impl Default for AtlasAllocator {
    fn default() -> Self {
        Self {
            free: vec![Rect {
                off: UVec2::ZERO,
                size: UVec2::new(MAX_W, MAX_H),
            }],
        }
    }
}

impl AtlasAllocator {
    /// Allocate space; returns `(layer, offset)` or `None` if full.
    pub fn allocate(&mut self, size: UVec2) -> Option<(u32, UVec2)> {
        let idx = self
            .free
            .iter()
            .position(|r| size.x <= r.size.x && size.y <= r.size.y)?;
        let rect = self.free.remove(idx);

        // right strip
        if rect.size.x > size.x {
            self.free.push(Rect {
                off: UVec2::new(rect.off.x + size.x, rect.off.y),
                size: UVec2::new(rect.size.x - size.x, size.y),
            });
        }
        // bottom strip
        if rect.size.y > size.y {
            self.free.push(Rect {
                off: UVec2::new(rect.off.x, rect.off.y + size.y),
                size: UVec2::new(rect.size.x, rect.size.y - size.y),
            });
        }
        Some((0, rect.off))
    }
}
