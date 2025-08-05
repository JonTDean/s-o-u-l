//! Bevy `Plugin` wrapper around the world-camera controller.

use bevy::prelude::*;

use crate::controls::camera::{
    controller::{
        CameraController, gather_input, apply_camera_controller
    },
    manager::{CameraSet, DragState},
};

/// Bundles the [`CameraController`] resource and its two systems.
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ resources -------------------------------------------------- */
        app.init_resource::<CameraController>()
           .init_resource::<DragState>();

        /* 2 ░ systems ---------------------------------------------------- */
        app.add_systems(
            Update,
            (
                gather_input.in_set(CameraSet::Input),
                apply_camera_controller.in_set(CameraSet::Heavy),
            ),
        );
    }
}
