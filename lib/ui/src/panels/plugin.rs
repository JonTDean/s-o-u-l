use bevy::prelude::*;

use crate::panels::{main_menu::MainMenuPlugin, world::WorldMenusPlugin};

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
               /* egui-based menu stack (main-menu, editors …)            */
               MainMenuPlugin,
               /* in-scene HUD overlays (automata list, minimap, pause …) */
               WorldMenusPlugin,
           ));
    }
}
