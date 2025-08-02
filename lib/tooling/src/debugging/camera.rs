use bevy::prelude::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Resource, Clone, Copy)]
    pub struct CameraDebug: u32 {
        const CLAMP       = 0b0000_0001;   // keep camera inside logical world
        const DRAW_BOUNDS = 0b0000_0010;   // draw world AABB
        const FRUSTUM     = 0b0000_0100;   // draw camera rectangle
        const FREEZE      = 0b0000_1000;   // ignore user input
        const LOG_SNAP    = 0b0001_0000;   // log floating-origin snaps
        const METRICS     = 0b0010_0000;   // print per-frame CameraMetrics
        const GRID_3D     = 0b0100_0000;   //  coloured axis grid
        const AXES        = 0b1000_0000;   // draw XYZ axes
        const FLOOR       = 0b1_0000_0000; // draw XY floor grid
    }
}
 
impl Default for CameraDebug {
    fn default() -> Self { Self::CLAMP }
}

/// Injects the [`CameraDebug`] resource so the render crate can
/// enable / disable individual flags at run-time (egui panel, F-keys, …).
pub struct CameraDebugPlugin;
impl Plugin for CameraDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDebug>();
    }
}
