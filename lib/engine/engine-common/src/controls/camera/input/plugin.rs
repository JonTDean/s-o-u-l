//! Facade plugin that wires the lightweight input helpers
//! (zoom / drag / pan / orbit) into the right [`CameraSet`].

use bevy::prelude::*;

use crate::controls::camera::{
    input::{
        drag::{begin_drag, drag_pan, end_drag},
        pan::key_pan,
        rotate::{apply_orbit, gather_orbit_input},
        zoom::{zoom_keyboard, zoom_scroll},
    },
    manager::{
        CameraSet, DragState,
        stage::{stage_keys_allowed, stage_pointer_allowed},
    },
};
use engine_core::prelude::AppState;

/// Camera Input Controls Plugin
pub struct InputFacadePlugin;

impl Plugin for InputFacadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (zoom_scroll, begin_drag, drag_pan, gather_orbit_input)
                    .run_if(stage_pointer_allowed),
                end_drag,
                (zoom_keyboard, key_pan).run_if(stage_keys_allowed),
            )
                .in_set(CameraSet::Input)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            apply_orbit
                .in_set(CameraSet::Heavy)
                .run_if(in_state(AppState::InGame)),
        )
        // ensure DragState exists even if the main controller plugin
        // is disabled (editor-only usage, headless tests…)
        .init_resource::<DragState>();
    }
}
