//! Injects the *Active Automata* HUD list (top-left) and
//! the “Spawner” window (top-right) while the game is running.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;

use crate::components::menus::automata::{
    active_menu::show_active_automata, spawn_menu::spawner::spawner,
};

pub struct AutomataMenusPlugin;

impl Plugin for AutomataMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            super::lenia::LeniaWorkbenchPlugin,
            super::active_menu::workbench::persistence::ProjectPlugin,
            super::active_menu::workbench::comparison::ComparisonPlugin,
        ));
        app.init_resource::<super::active_menu::MinimapSelection>();
        app.add_systems(
            EguiPrimaryContextPass,
            (
                super::active_menu::workbench::show.run_if(in_state(AppState::InGame)),
                // HUD list
                show_active_automata.run_if(in_state(AppState::InGame)),
                // Spawn-pattern panel
                spawner.run_if(in_state(AppState::InGame)),
            ),
        );
    }
}
