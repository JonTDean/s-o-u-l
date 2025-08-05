//! Debug-drawing helpers for camera frustum & automata bounds.
//
//! • Clears the gizmo layer deterministically *once per frame* whenever
//!   we are about to (re)draw either the frustum or the automata bounds.
//!   This removes the “right-hand quadrant” artefact that was caused by
//!   stale geometry accumulating every frame.

use bevy::prelude::*;
use engine_core::prelude::AutomataRegistry;
use tooling::debugging::camera::CameraDebug;

use crate::render::camera::systems::{ViewportRect, WorldCamera};

pub fn draw_camera_gizmos(
    windows:  Query<&Window>,
    flags:    Res<CameraDebug>,
    cam_q:    Query<(&GlobalTransform, &Projection, &ViewportRect), With<WorldCamera>>,
    registry: Res<AutomataRegistry>,
    mut gizmos: Gizmos,
) {
    /* ───── determine whether we need to draw anything this frame ───── */
    let wants_frustum = flags.contains(CameraDebug::FRUSTUM);
    let wants_bounds  = flags.contains(CameraDebug::DRAW_BOUNDS);
    let active        = wants_frustum || wants_bounds;

    /* ───── when nothing is active, wipe the old gizmos and bail out ── */
    if !active {
        gizmos.clear();
        return;
    }

    /* ───── we *will* draw – start with a clean slate every frame ───── */
    gizmos.clear();

    /* ───── fetch camera + window ───────────────────────────────────── */
    let (Ok((xf, Projection::Orthographic(o), rect)), Ok(win)) =
        (cam_q.single(), windows.single())
    else { return };

    let half_w = win.width()  * 0.5 * o.scale;
    let half_h = win.height() * 0.5 * o.scale;
    let c      = xf.translation();

    /* ───────────────────── camera frustum ──────────────────────────── */
    if wants_frustum {
        let r = [
            Vec3::new(rect.min.x, rect.min.y, 0.0),
            Vec3::new(rect.max.x, rect.min.y, 0.0),
            Vec3::new(rect.max.x, rect.max.y, 0.0),
            Vec3::new(rect.min.x, rect.max.y, 0.0),
        ];
        for i in 0..4 {
            gizmos.line(r[i], r[(i + 1) % 4], Color::hsla(240.0, 1.0, 0.5, 1.0));
        }
    }

    /* ─────────────────── automata world bounds ─────────────────────── */
    if wants_bounds {
        if registry.list().is_empty() { return; }

        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for info in registry.list() {
            let off  = info.world_offset;
            let size = Vec3::new(
                info.slice.size.x as f32,
                info.slice.size.y as f32,
                0.0,
            ) * info.voxel_size;
            min = min.min(off);
            max = max.max(off + size);
        }

        /* ensure the box spans at least one viewport so it never vanishes */
        min.x = min.x.min(c.x - half_w);
        min.y = min.y.min(c.y - half_h);
        max.x = max.x.max(c.x + half_w);
        max.y = max.y.max(c.y + half_h);

        let r = [
            Vec3::new(min.x, min.y, 0.0),
            Vec3::new(max.x, min.y, 0.0),
            Vec3::new(max.x, max.y, 0.0),
            Vec3::new(min.x, max.y, 0.0),
        ];
        for i in 0..4 {
            gizmos.line(r[i], r[(i + 1) % 4], Color::hsla(60.0, 1.0, 0.5, 1.0));
        }
    }
}
