//! Debug‑drawing helpers for camera frustum & world bounds.

use bevy::prelude::*;
use simulation_kernel::grid::GridBackend;
use tooling::debugging::camera::CameraDebug;

use crate::render::{
    camera::systems::{dynamic_world_size, WorldCamera},
    worldgrid::WorldGrid,
};

pub fn draw_camera_gizmos(
    windows: Query<&Window>,
    flags:   Res<CameraDebug>,
    grid:    Option<Res<WorldGrid>>,
    cam_q:   Query<(&GlobalTransform, &Projection), With<WorldCamera>>,
    mut gizmos: Gizmos,
) {
    // Clear when toggling off.
    if flags.is_changed() && !flags.intersects(CameraDebug::DRAW_BOUNDS | CameraDebug::FRUSTUM) {
        gizmos.clear();
    }

    /* world bounds */
    if flags.contains(CameraDebug::DRAW_BOUNDS) {
        let Some(grid) = grid else { return };
        let (w, h) = match &grid.backend {
            GridBackend::Dense(g)  => {
                let (Ok(win), Ok((_xf, Projection::Orthographic(o)))) =
                    (windows.single(), cam_q.single()) else { return };
                let v = dynamic_world_size(win, o.scale, g);
                (v.x, v.y)
            },
            GridBackend::Sparse(_) => (1_024.0, 1_024.0),
        };
        let rect = [
            Vec3::ZERO,
            Vec3::new(w, 0.0, 0.0),
            Vec3::new(w, h, 0.0),
            Vec3::new(0.0, h, 0.0),
        ];
        for i in 0..4 {
            gizmos.line(rect[i], rect[(i + 1) % 4], Color::srgb(0.0, 1.0, 0.0));
        }
    }

    /* camera frustum */
    if flags.contains(CameraDebug::FRUSTUM) {
        let (Ok((xf, Projection::Orthographic(o))), Ok(win)) = (cam_q.single(), windows.single()) else { return };
        let hw = win.width() * 0.5 * o.scale;
        let hh = win.height() * 0.5 * o.scale;
        let c  = xf.translation();
        let rect = [
            Vec3::new(c.x - hw, c.y - hh, 0.0),
            Vec3::new(c.x + hw, c.y - hh, 0.0),
            Vec3::new(c.x + hw, c.y + hh, 0.0),
            Vec3::new(c.x - hw, c.y + hh, 0.0),
        ];
        for i in 0..4 {
            gizmos.line(rect[i], rect[(i + 1) % 4], Color::srgb(1.0, 1.0, 0.0));
        }
    }
}