//! Glue layer that spawns/destroys the `NewScenarioMenuScreen` resource and
//! drives its UI when the app is in `AppState::NewScenario`.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;

use crate::components::menus::{menu_runner, scenarios_menu::new::NewScenarioMenuScreen};


pub struct NewScenarioMenuUiPlugin;

impl Plugin for NewScenarioMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::NewScenario), |mut cmd: Commands| {
                cmd.insert_resource(NewScenarioMenuScreen::default());
            })
            .add_systems(
                EguiPrimaryContextPass,
                menu_runner::<NewScenarioMenuScreen>.run_if(in_state(AppState::NewScenario)),
            )
            .add_systems(OnExit(AppState::NewScenario), |mut cmd: Commands| {
                cmd.remove_resource::<NewScenarioMenuScreen>();
            });
    }
}
