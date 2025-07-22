//! Root plugin for *in‑game* (world) HUD overlays.
//!
//! At the moment it bundles only the **Active Automata** panel, but new HUD
//! widgets (FPS graphs, minimaps, etc.) can be added here later.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::state::AppState;
use crate::ui::panels::world::{pause_menu::PauseMenuPlugin, zoom_overlay::zoom_overlay};

use super::automata::AutomataPanelPlugin;     // ← only the existing plugin

pub struct WorldMenusPlugin;
impl Plugin for WorldMenusPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((AutomataPanelPlugin, PauseMenuPlugin))
            .add_systems(
                EguiPrimaryContextPass,
                zoom_overlay.run_if(in_state(AppState::InGame)),
            );

    }
}
