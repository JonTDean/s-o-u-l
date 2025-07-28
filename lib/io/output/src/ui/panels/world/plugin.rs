//! Root plugin for *in‑game* (world) HUD overlays.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::state::AppState;

use crate::ui::panels::world::minimap_overlay::MinimapSelection;

use super::{
    automata::AutomataPanelPlugin,
    pause_menu::PauseMenuPlugin,
    zoom_overlay::zoom_overlay,
    minimap_overlay::minimap_overlay, 
};

pub struct WorldMenusPlugin;

impl Plugin for WorldMenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapSelection>();
        app.add_plugins((AutomataPanelPlugin, PauseMenuPlugin))
            // HUD widgets run in the egui pass only while playing.
            .add_systems(
                EguiPrimaryContextPass,
                (zoom_overlay, minimap_overlay)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
