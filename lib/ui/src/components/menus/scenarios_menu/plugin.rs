use bevy::app::{App, Plugin};

use crate::components::menus::scenarios_menu::{
    load::plugin::LoadScenarioMenuUiPlugin, 
    new::plugin::NewScenarioMenuUiPlugin
};

pub struct SenariosMenuUiPlugin;

impl Plugin for SenariosMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NewScenarioMenuUiPlugin,
            LoadScenarioMenuUiPlugin,
        ));
    }
}
