//! Root plugin for *all* in-game HUD overlays.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;

use crate::panels::world::camera_overlays::camera_debug::debug_camera_menu;

use super::{
    automata::AutomataPanelPlugin,
    minimap_overlay::{minimap_overlay, MinimapSelection},
    pause_menu::PauseMenuPlugin,
    camera_overlays::camera_overlay,
};

pub struct WorldMenusPlugin;

impl Plugin for WorldMenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapSelection>()
            .add_plugins((AutomataPanelPlugin, PauseMenuPlugin))
            .add_systems(
                EguiPrimaryContextPass,
                (camera_overlay, minimap_overlay, debug_camera_menu)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
