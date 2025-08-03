use bevy::prelude::*;
use super::camera::CameraDebug;

///  unit-length axis colours (matches Blender)
pub const RED:   Color = Color::srgb(1.0, 0.0, 0.0);
pub const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
pub const BLUE:  Color = Color::srgb(0.0, 0.0, 1.0);
const GREY: Color = Color::srgb(0.3, 0.3, 0.3);

pub fn draw_axes_and_floor(
    flags:  Res<CameraDebug>,
    mut giz: Gizmos,
) {
    if flags.is_changed() && !flags.intersects(CameraDebug::AXES | CameraDebug::FLOOR_GRID) {
        giz.clear();
    }
    
    /* floor grid – grey 1 m spacing, 1 km extent */
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
    
    /* axis lines – 1 km long, Blender colours */
    if flags.contains(CameraDebug::AXES) {
        let len = 1000.0;
        giz.line(Vec3::ZERO, Vec3::X * len, Color::srgb(1.0,0.0,0.0)); // X – red
        giz.line(Vec3::ZERO, Vec3::Y * len, Color::srgb(0.0,1.0,0.0)); // Y – green
        giz.line(Vec3::ZERO, Vec3::Z * len, Color::srgb(0.0,0.0,1.0)); // Z – blue
    }
}