//! Glue layer for the *Load Scenario* menu.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;
use engine_common::scenes::scenarios::load::plugin::ScenarioManifest;

use crate::components::menus::{menu_runner, scenarios_menu::load::LoadScenarioMenuScreen};


pub struct LoadScenarioMenuUiPlugin;

impl Plugin for LoadScenarioMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadScenario),
            |mut cmd: Commands, manifest: Res<ScenarioManifest>| {
                cmd.insert_resource(LoadScenarioMenuScreen::new(manifest.files.clone()));
            },
        )
        .add_systems(
            EguiPrimaryContextPass,
            menu_runner::<LoadScenarioMenuScreen>.run_if(in_state(AppState::LoadScenario)),
        )
        .add_systems(OnExit(AppState::LoadScenario), |mut cmd: Commands| {
            cmd.remove_resource::<LoadScenarioMenuScreen>();
        });
    }
}
