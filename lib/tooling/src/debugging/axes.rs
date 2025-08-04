//! tooling/debugging/axes.rs
//! ---------------------------------------------------------------------------
//! **Bug‑fix (2025‑08‑03)** – Prevents the 3‑D voxel grid from disappearing
//! when you toggle it via the debug menu.
//!
//! The original logic cleared *all* gizmos whenever any flag changed **except**
//! `AXES` or `FLOOR_GRID`.  Toggling `GRID_3D` therefore wiped the freshly
//! drawn grid in the very next system pass, making it look as if nothing
//! happened unless `FLOOR_GRID` was also enabled.
//!
//! We fix this by evaluating the union of “line‑drawing” flags *inside the
//! function* (avoiding const‑expression limitations with `bitflags`) and only
//! clearing when **none** of them are active.
//!
//! Additionally, we migrate colour helpers to `Color::rgb_u8`.

use bevy::prelude::*;
use super::camera::CameraDebug;

/// Unit‑axis colours (matches Blender).
pub const RED:   Color = Color::srgb_u8(255, 0, 0);
pub const GREEN: Color = Color::srgb_u8(0, 255, 0);
pub const BLUE:  Color = Color::srgb_u8(0, 0, 255);
const GREY:      Color = Color::srgb_u8(77, 77, 77);

pub fn draw_axes(
    flags:  Res<CameraDebug>,
    mut giz: Gizmos,
) {
    /* ── smart clearing ──────────────────────────────────────────────── */
    // Combine the flags that draw line gizmos each frame. We compute this
    // at run‑time to avoid `const` limitations with `bitflags` (the `|`
    // operator isn’t a const‑fn).
    let line_flags = CameraDebug::AXES | CameraDebug::FLOOR_GRID | CameraDebug::GRID_3D;

    if flags.is_changed() && !flags.intersects(line_flags) {
        giz.clear();
    }

    /* ── grey XY floor grid (1 m spacing, ±500 m extent) ───────────── */
    if flags.contains(CameraDebug::FLOOR_GRID) {
        let step = 1.0;
        let half = 500.0;
        for x in (-half as i32..=half as i32).map(|n| n as f32 * step) {
            giz.line(Vec3::new(x, -half, 0.0), Vec3::new(x,  half, 0.0), GREY);
        }
        for y in (-half as i32..=half as i32).map(|n| n as f32 * step) {
            giz.line(Vec3::new(-half, y, 0.0), Vec3::new( half, y, 0.0), GREY);
        }
    }

    /* ── XYZ axes (1 km length) ─────────────────────────────────────── */
    if flags.contains(CameraDebug::AXES) {
        let len = 1_000.0;
        giz.line(Vec3::ZERO, Vec3::X * len, RED);   // X – red
        giz.line(Vec3::ZERO, Vec3::Y * len, GREEN); // Y – green
        giz.line(Vec3::ZERO, Vec3::Z * len, BLUE);  // Z – blue
    }
}
