//! Glue layer that spawns/destroys the `MainMenuScreen` resource and
//! runs its UI when the app is in `AppState::MainMenu`.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;

use crate::components::menus::{menu_runner, meta::main_menu::MainMenuScreen};



pub struct MainMenuUiPlugin;

impl Plugin for MainMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), |mut cmd: Commands| {
                cmd.insert_resource(MainMenuScreen::default());
            })
            .add_systems(
                EguiPrimaryContextPass,
                menu_runner::<MainMenuScreen>.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), |mut cmd: Commands| {
                cmd.remove_resource::<MainMenuScreen>();
            });
    }
}
