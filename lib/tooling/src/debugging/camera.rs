use bevy::prelude::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Resource, Clone, Copy)]
    pub struct CameraDebug: u32 {
        const CLAMP       = 0b00000001;   // keep hard bounds
        const DRAW_BOUNDS = 0b00000010;   // draw grid AABB
        const FRUSTUM     = 0b00000100;   // draw camera rectangle
        const FREEZE      = 0b00001000;   // ignore user input
        const LOG_SNAP    = 0b00010000;   // print floating-origin snaps
    }
}

impl Default for CameraDebug {
    fn default() -> Self { Self::CLAMP }        // only clamp enabled by default
}

pub struct CameraDebugPlugin;
impl Plugin for CameraDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDebug>(); 
    }
}