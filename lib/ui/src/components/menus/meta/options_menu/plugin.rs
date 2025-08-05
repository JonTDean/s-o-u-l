use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::AppState;

use crate::components::menus::{menu_runner, meta::options_menu::{commit_on_exit, OptionsMenu}};


pub struct OptionsMenuUiPlugin;

impl Plugin for OptionsMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Options), |mut cmd: Commands| {
                cmd.init_resource::<OptionsMenu>();   // calls FromWorld
            })
            .add_systems(
                EguiPrimaryContextPass,
                menu_runner::<OptionsMenu>.run_if(in_state(AppState::Options)),
            )
            .add_systems(OnExit(AppState::Options), (
                commit_on_exit,
                |mut cmd: Commands| cmd.remove_resource::<OptionsMenu>(),
            ));
    }
}
