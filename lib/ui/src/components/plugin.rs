use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;
use tooling::debugging::{axes::draw_axes_and_floor, camera::CameraDebugPlugin, grid::draw_3d_grid};
use bevy::transform::TransformSystem;

use super::{
    camera::{camera_overlay, camera_debug::debug_camera_menu},
    minimap::minimap_overlay,
};

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BannersPlugin); 
    }
}
