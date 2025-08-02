//! Camera stack: UI + world cameras, input controls and floating-origin.

mod gizmos;
mod input;

pub(crate) mod floating_origin;
pub mod systems;
pub mod controller;
pub mod freecam;

use bevy::prelude::*;
use engine_core::prelude::*;

use systems::{spawn_cameras, DragState};
use crate::render::camera::systems::CameraSet;

use self::floating_origin::WorldOffset;   // local module path

/* ─────────────────── Public re-exports ──────────────────────────── */
pub use systems::{WorldCamera, ZoomInfo}; 

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
               )
               .run_if(in_state(AppState::InGame)),
           );

        app.add_systems(
        Update,
        gizmos::draw_camera_gizmos
            .run_if(in_state(AppState::InGame))
            .in_set(CameraSet::Heavy),   // keep the original ordering
        );
    }
}
