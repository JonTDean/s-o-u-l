// lib/tooling/src/debugging/grid.rs
use bevy::prelude::*;
use engine_core::prelude::AutomataRegistry;
use super::camera::CameraDebug;

/* colour helpers – Bevy has no RED/GREEN/BLUE constants */
const GRID_CLR_X: Color = Color::srgb(1.0, 0.0, 0.0);
const GRID_CLR_Y: Color = Color::srgb(0.0, 1.0, 0.0);
const GRID_CLR_Z: Color = Color::srgb(0.0, 0.0, 1.0);

const STEP: usize = 4;
const CAP:  usize = 32;

/// Draw axis-coloured voxel grids when `GRID_3D` is enabled.
pub fn draw_3d_grid(
    flags:     Res<CameraDebug>,
    registry:  Res<AutomataRegistry>,
    mut giz:   Gizmos,
) {
    if !flags.contains(CameraDebug::GRID_3D) { return; }
    giz.clear();

    let z_bias = -0.05; // fixed offset – we no longer need the camera transform

    for info in registry.list() {
        let off  = info.world_offset;
        let vox  = info.voxel_size.max(f32::EPSILON);
        let sz   = info.slice.size;

        let max = Vec2::new(sz.x as f32 * vox, sz.y as f32 * vox);

        /* Z pillars */
        for c in [Vec2::ZERO, Vec2::new(max.x,0.0), Vec2::new(0.0,max.y), max] {
            giz.line(
                (off.xy()+c).extend(z_bias),
                (off.xy()+c).extend(z_bias + sz.y as f32 * vox),
                GRID_CLR_Z,
            );
        }

        /* XY layers */
        for l in (0..=sz.y.min(CAP as u32)).step_by(STEP) {
            let z = z_bias + l as f32 * vox;

            // Y-axis (green)
            for x in 0..=sz.x {
                let wx = off.x + x as f32 * vox;
                giz.line(
                    Vec3::new(wx, off.y,       z),
                    Vec3::new(wx, off.y+max.y, z),
                    GRID_CLR_Y,
                );
            }
            // X-axis (red)
            for y in 0..=sz.y {
                let wy = off.y + y as f32 * vox;
                giz.line(
                    Vec3::new(off.x,     wy, z),
                    Vec3::new(off.x+max.x, wy, z),
                    GRID_CLR_X,
                );
            }
        }
    }
}
