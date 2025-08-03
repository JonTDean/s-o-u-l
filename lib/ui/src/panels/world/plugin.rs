//! Root plugin for *all* in-game HUD overlays.

use bevy::prelude::*;

use super::{
    automata::AutomataPanelPlugin,
    pause_menu::PauseMenuPlugin,
};

pub struct WorldMenusPlugin;

impl Plugin for WorldMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                    AutomataPanelPlugin, 
                    PauseMenuPlugin,
                ));
    }
}
