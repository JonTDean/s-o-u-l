// lib/tooling/src/debugging/grid.rs
//! 3-D debug voxel grid gizmo.
//!
//! * Clears every frame before drawing (prevents line build-up).
//! * Draws only every `STEP`-th voxel line to keep the view readable.
//! * Respects each automaton’s `world_offset` so grids appear in place.

use bevy::prelude::*;
use engine_core::prelude::AutomataRegistry;

use super::camera::CameraDebug;

/* ──────────────── colour helpers ──────────────── */
const GRID_CLR_X: Color = Color::srgb_u8(255, 0, 0);   // red   → X axis
const GRID_CLR_Y: Color = Color::srgb_u8(0, 255, 0);   // green → Y axis
const GRID_CLR_Z: Color = Color::srgb_u8(0, 0, 255);   // blue  → Z axis

/* ─────────────── grid density limits ───────────── */
const STEP: usize = 4;    // draw every N-th voxel line
const CAP:  usize = 32;   // maximum layers per automaton

/// Draws axis-coloured voxel grids for every registered automaton.
pub fn draw_3d_grid(
    flags:     Res<CameraDebug>,
    registry:  Res<AutomataRegistry>,
    mut giz:   Gizmos,
) {
    /* ───── early outs ───── */
    if !flags.contains(CameraDebug::GRID_3D)          { return; }
    if registry.list().is_empty()                     { return; }

    /* fresh frame */
    giz.clear();

    const Z_BIAS: f32 = -0.05; // avoid z-fighting with slices

    for info in registry.list() {
        let off = info.world_offset;
        let vox = info.voxel_size.max(f32::EPSILON);
        let sz  = info.slice.size;

        let max = Vec2::new(sz.x as f32 * vox, sz.y as f32 * vox) + off.truncate();

        /* ───── pillars (vertical) ───── */
        for corner in [
            off.truncate(),
            Vec2::new(max.x, off.y),
            Vec2::new(off.x, max.y),
            max,
        ] {
            giz.line(
                corner.extend(Z_BIAS + off.z),
                corner.extend(Z_BIAS + off.z + sz.y as f32 * vox),
                GRID_CLR_Z,
            );
        }

        /* ───── XY layers ───── */
        for l in (0..=sz.y.min(CAP as u32) as usize).step_by(STEP) {
            let z = Z_BIAS + off.z + l as f32 * vox;

            /* vertical (parallel Y) */
            for x in (0..=sz.x as usize).step_by(STEP) {
                let wx = off.x + x as f32 * vox;
                giz.line(
                    Vec3::new(wx, off.y, z),
                    Vec3::new(wx, max.y, z),
                    GRID_CLR_Y,
                );
            }

            /* horizontal (parallel X) */
            for y in (0..=sz.y as usize).step_by(STEP) {
                let wy = off.y + y as f32 * vox;
                giz.line(
                    Vec3::new(off.x, wy, z),
                    Vec3::new(max.x, wy, z),
                    GRID_CLR_X,
                );
            }
        }
    }
}
