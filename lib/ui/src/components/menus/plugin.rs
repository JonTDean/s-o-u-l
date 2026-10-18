use bevy::prelude::*;

use crate::components::menus::{
    automata::plugin::AutomataMenusPlugin, meta::plugin::MetaMenusPlugin,
};

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MetaMenusPlugin, AutomataMenusPlugin));
    }
}
