//! Injects the *Active Automata* HUD list (top-left) and
//! the “Spawner” window (top-right) while the game is running.

use bevy::prelude::*;

use crate::components::menus::meta::{main_menu, options_menu, scenarios_menu};


pub struct MetaMenusPlugin;

impl Plugin for MetaMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_menu::plugin::MainMenuUiPlugin,
            options_menu::plugin::OptionsMenuUiPlugin,
            scenarios_menu::plugin::SenariosMenuUiPlugin,
        ));
    }
}
