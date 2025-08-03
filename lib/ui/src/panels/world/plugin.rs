//! Root plugin for *all* in-game HUD overlays.

use bevy::prelude::*;
use engine_render::render::minimap::MinimapTextures;

use crate::overlays::minimap::MinimapSelection;

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
