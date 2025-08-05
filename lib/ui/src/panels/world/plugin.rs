//! Root plugin for *all* in-game HUD overlays.

use bevy::prelude::*;

use super::{
    pause_menu::PauseMenuPlugin,
};

pub struct WorldMenusPlugin;

impl Plugin for WorldMenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                    PauseMenuPlugin,
                ));
    }
}
