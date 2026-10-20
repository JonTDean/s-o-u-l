//! Full-screen meta menus: main, options, scenarios and the pause menu.

use bevy::prelude::*;

use crate::components::menus::meta::{main_menu, options_menu, pause_menu, scenarios_menu};

pub struct MetaMenusPlugin;

impl Plugin for MetaMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_menu::plugin::MainMenuUiPlugin,
            options_menu::plugin::OptionsMenuUiPlugin,
            scenarios_menu::plugin::SenariosMenuUiPlugin,
            pause_menu::PauseMenuPlugin,
        ));
    }
}
