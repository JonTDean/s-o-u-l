//! overlays/plugin.rs – bundles every run-time overlay & debug gizmo.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;
use tooling::debugging::{
    axes::draw_axes_and_floor,
    camera::CameraDebugPlugin,
    grid::draw_3d_grid,
};
use bevy::transform::TransformSystem;

use super::{
    camera::{camera_overlay, camera_debug::debug_camera_menu},
    minimap::minimap_overlay,
    debug_stats::debug_stats_panel,
};

/// Master overlay plugin (in-game HUD, gizmos, debug menus).
pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        /* debug-flag resource */
        app.add_plugins(CameraDebugPlugin);

        /* egui widgets – run only in-game */
        app.add_systems(
            EguiPrimaryContextPass,
            (
                camera_overlay,
                minimap_overlay,
                debug_camera_menu,   // F3
                debug_stats_panel,   // F4
            )
            .run_if(in_state(AppState::InGame)),
        );

        /* wire-frame gizmos */
        app.add_systems(
            Update,
            (
                draw_3d_grid,
                draw_axes_and_floor,
            )
            .run_if(in_state(AppState::InGame))
            .after(TransformSystem::TransformPropagate),
        );
    }
}
