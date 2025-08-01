//! Injects the *Active Automata* HUD list (top-left) and
//! the “Spawner” window (top-right) while the game is running.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;

use crate::panels::world::automata::{selection_arrow::SelectionArrowPlugin, show_active_automata::show_active_automata, spawn_panel::spawn_panel};

pub struct AutomataPanelPlugin;

impl Plugin for AutomataPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass,
        (
                // HUD list
                show_active_automata.run_if(in_state(AppState::InGame)),
                // Spawn-pattern panel
                spawn_panel.run_if(in_state(AppState::InGame)),
        ))
        .add_plugins(SelectionArrowPlugin);
    }
}
