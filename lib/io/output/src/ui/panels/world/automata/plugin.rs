//! Injects the *Active Automata* overlay + spawner window while in‑game.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::ui::panels::world::automata::{
    show_active_automata::show_active_automata,
    spawn_panel,
};
use engine_core::state::AppState;

pub struct AutomataPanelPlugin;

impl Plugin for AutomataPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                EguiPrimaryContextPass,
                (
                    // HUD list (top‑left)
                    show_active_automata
                        .run_if(in_state(AppState::InGame)),
                    // Spawner window (top‑right)
                    spawn_panel::spawn_panel
                        .run_if(in_state(AppState::InGame)),
                ),
            );
    }
}
