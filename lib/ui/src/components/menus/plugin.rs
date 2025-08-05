use bevy::prelude::*;

use crate::components::menus::{
    meta::plugin::MetaMenusPlugin,
    automata::plugin::AutomataMenusPlugin
};

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MetaMenusPlugin,
            AutomataMenusPlugin,
        ));
    }
}

