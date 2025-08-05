//! Facade plugin that wires the lightweight input helpers
//! (zoom / drag / pan / orbit) into the right [`CameraSet`].

use bevy::prelude::*;

use crate::controls::camera::{
    input::{
        zoom::{zoom_scroll, zoom_keyboard},
        drag::{begin_drag, drag_pan, end_drag},
        pan::key_pan,
        rotate::{gather_orbit_input, apply_orbit},
    },
    manager::{CameraSet, DragState},
};
use engine_core::prelude::AppState;

/// Camera Input Controls Plugin
pub struct InputFacadePlugin;

impl Plugin for InputFacadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                zoom_scroll,
                zoom_keyboard,
                begin_drag,
                drag_pan,
                end_drag,
                key_pan,
                gather_orbit_input,
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
