//! `Plugin` for the optional perspective free-camera.

use bevy::prelude::*;

use crate::controls::camera::{
    freecam::{
        FreeCamSettings, gather_keyboard_input, gather_mouse_input,
        apply_freecam_motion
    },
    manager::CameraSet,
};
use engine_core::systems::state::AppState;

/// Drop-in free-cam plugin (does nothing until you `spawn_freecam()`).
pub struct FreeCamPlugin;

impl Plugin for FreeCamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FreeCamSettings>()
            /* light input collection ------------------------------------ */
            .add_systems(
                Update,
                (
                    gather_keyboard_input.in_set(CameraSet::Input),
                    gather_mouse_input.in_set(CameraSet::Input),
                )
                .run_if(in_state(AppState::InGame)),
            )
            /* heavy integrator ------------------------------------------ */
            .add_systems(
                Update,
                apply_freecam_motion
                    .in_set(CameraSet::Heavy)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
