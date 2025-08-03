use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;
use tooling::debugging::{axes::draw_axes_and_floor, camera::CameraDebugPlugin, grid::draw_3d_grid};
use bevy::transform::TransformSystem;

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct BannersPlugin;

impl Plugin for BannersPlugin {
    fn build(&self, app: &mut App) {
        /* resources + flag plugin */
        app.add_plugins(CameraDebugPlugin)
           /* egui widgets */
           .add_systems(
               EguiPrimaryContextPass,
               (
                   camera_overlay,
                   minimap_overlay,
                   debug_camera_menu,   // ← F3 window
               )
                   .run_if(in_state(AppState::InGame)),
           )
           /* gizmo-based debug drawings */
            .add_systems(
                Update,
                (
                    draw_3d_grid,
                    draw_axes_and_floor,
                )
                .run_if(in_state(AppState::InGame))
                .after(TransformSystem::TransformPropagate),
            );
        app.add_plugins(PauseBannerPlugin); 
    }
}
