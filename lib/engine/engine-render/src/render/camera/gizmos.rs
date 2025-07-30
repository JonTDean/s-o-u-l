//! Debug-drawing helpers for the camera frustum & grid bounds.

use bevy::prelude::*;
use simulation_kernel::grid::GridBackend;
use tooling::debugging::camera::CameraDebug;

use crate::render::{
    camera::systems::{WorldCamera},
    worldgrid::WorldGrid,
};

/// Draws the grid AABB and/or camera frustum when the corresponding
/// `CameraDebug` flags are enabled.
pub fn draw_camera_gizmos(
    windows:  Query<&Window>,
    flags:    Res<CameraDebug>,
    grid:     Option<Res<WorldGrid>>,
    cam_q:    Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut gizmos: Gizmos,
) {
    /* wipe when we turn drawing *off* */
    if flags.is_changed() && !flags.intersects(CameraDebug::DRAW_BOUNDS | CameraDebug::FRUSTUM) {
        gizmos.clear();
    }

    /* 1 ░ grid AABB --------------------------------------------------- */
    if flags.contains(CameraDebug::DRAW_BOUNDS) {
        let Some(grid) = grid else { return };
        let (w, h) = match &grid.backend {
            GridBackend::Dense(g)  => (g.size.x as f32, g.size.y as f32),
            GridBackend::Sparse(_) => (1_024.0, 1_024.0),
        };
        let c0 = Vec3::new(0.0, 0.0, 0.0);
        let c1 = Vec3::new(w,   0.0, 0.0);
        let c2 = Vec3::new(w,   h,   0.0);
        let c3 = Vec3::new(0.0, h,   0.0);
        for (a, b) in [(c0, c1), (c1, c2), (c2, c3), (c3, c0)] {
            gizmos.line(a, b, Color::srgb(0.0, 1.0, 0.0));
        }
    }

    /* 2 ░ camera frustum --------------------------------------------- */
    if flags.contains(CameraDebug::FRUSTUM) {
        let Ok((xf, Projection::Orthographic(o))) = cam_q.single() else { return };
        let Ok(win) = windows.single()                             else { return };
        let hw = win.width()  * 0.5 * o.scale;
        let hh = win.height() * 0.5 * o.scale;
        let c  = xf.translation();
        let p0 = Vec3::new(c.x - hw, c.y - hh, 0.0);
        let p1 = Vec3::new(c.x + hw, c.y - hh, 0.0);
        let p2 = Vec3::new(c.x + hw, c.y + hh, 0.0);
        let p3 = Vec3::new(c.x - hw, c.y + hh, 0.0);
        for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
            gizmos.line(a, b, Color::srgb(1.0, 1.0, 0.0));
        }
    }
}
