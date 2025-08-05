use bevy::prelude::*;

use crate::components::menus::{main_menu, options_menu, scenarios_menu};

pub struct MenusPlugin;        // add once, forget forever

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_menu::plugin::MainMenuUiPlugin,
            options_menu::plugin::OptionsMenuUiPlugin,
            scenarios_menu::plugin::SenariosMenuUiPlugin,
        ));
    }
}

