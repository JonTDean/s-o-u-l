//! Camera stack: UI + world cameras, input controls and floating-origin.

mod gizmos;
mod input;
mod floating_origin;
pub mod systems;

use bevy::prelude::*;
use engine_core::prelude::*;

use systems::{spawn_cameras, DragState};
use self::floating_origin::WorldOffset;   // local module path

/* ─────────────────── Public re-exports ──────────────────────────── */
pub use systems::{WorldCamera, ZoomInfo, UI_LAYER, WORLD_LAYER}; 

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomInfo>()
           .init_resource::<WorldOffset>()
           .init_resource::<DragState>()
           .add_systems(Startup, spawn_cameras)
           .add_systems(
               Update,
               (
                   input::zoom_scroll,
                   input::zoom_keyboard,
                   input::begin_drag,
                   input::drag_pan,
                   input::end_drag,
                   input::key_pan,
                   floating_origin::apply_floating_origin,
                   gizmos::draw_camera_gizmos,
               )
               .run_if(in_state(AppState::InGame)),
           );
    }
}
