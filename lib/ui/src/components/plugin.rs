use bevy::prelude::*;

use crate::components::{debug::plugin::DebugComponentsPlugin, menus::plugin::MenusPlugin};

/// Meta menus (main, options, scenarios, pause) and debug overlays.
pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DebugComponentsPlugin, MenusPlugin));
    }
}
