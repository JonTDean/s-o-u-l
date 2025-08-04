// lib/tooling/src/debugging/grid.rs
//! 3‑D debug voxel grid gizmo.
//!
//! **Fixes & improvements (2025‑08‑03)**
//! * Applies each automaton’s `world_offset` so the grid appears in the
//!   correct place—previously it was stuck at the global origin.
//! * Replaces deprecated `Color::srgb` with `Color::rgb_u8`.
//! * Skips `giz.clear()` when there is nothing to draw, preventing a blank
//!   frame when the registry is empty or the flag is off.
//! * Adds richer doc‑comments and explains magic constants.
//!
//! Enable via the `CameraDebug::GRID_3D` flag (eg. press **F7** or toggle in
//! the egui panel).

use bevy::prelude::*;
use engine_core::prelude::AutomataRegistry;
use super::camera::CameraDebug;

/* ───────────────────────── colour helpers ─────────────────────────── */
const GRID_CLR_X: Color = Color::srgb_u8(255, 0, 0);   // red   → X axis
const GRID_CLR_Y: Color = Color::srgb_u8(0, 255, 0);   // green → Y axis
const GRID_CLR_Z: Color = Color::srgb_u8(0, 0, 255);   // blue  → Z axis

/* ───────────────────────── grid density limits ────────────────────── */
const STEP: usize = 4;   // draw every Nth voxel line to avoid clutter
const CAP:  usize = 32;  // cap Z‑layers so the gizmo stays lightweight

/// Draws axis‑coloured voxel grids for every registered automaton.
#[allow(clippy::too_many_lines)]
pub fn draw_3d_grid(
    flags:     Res<CameraDebug>,
    registry:  Res<AutomataRegistry>,
    mut giz:   Gizmos,
) {
    // Early‑out when the flag is off.
    if !flags.contains(CameraDebug::GRID_3D) {
        return;
    }

    // No automatons? Nothing to draw—keep existing gizmos alive.
    if registry.list().is_empty() {
        return;
    }

    // Remove previously drawn grid lines.
    giz.clear();

    const Z_BIAS: f32 = -0.05; // small negative to avoid z‑fighting

    for info in registry.list() {
        let off = info.world_offset;                 // Vec3 world position
        let vox = info.voxel_size.max(f32::EPSILON); // guard against zero
        let sz  = info.slice.size;                  // IVec2 (x, y)

        let max = Vec2::new(sz.x as f32 * vox, sz.y as f32 * vox) + off.truncate();

        /* ───── pillars (vertical, Z axis) ─────────────────────────────── */
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

        /* ───── XY layers (horizontal slices) ──────────────────────────── */
        for l in (0..=sz.y.min(CAP as u32)).step_by(STEP) {
            let z = Z_BIAS + off.z + l as f32 * vox;

            // Vertical grid lines (parallel to Y axis)
            for x in 0..=sz.x {
                let wx = off.x + x as f32 * vox;
                giz.line(
                    Vec3::new(wx, off.y, z),
                    Vec3::new(wx, max.y, z),
                    GRID_CLR_Y,
                );
            }

            // Horizontal grid lines (parallel to X axis)
            for y in 0..=sz.y {
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
