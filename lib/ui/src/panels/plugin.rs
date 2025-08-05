use bevy::prelude::*;

use crate::panels::world::WorldMenusPlugin;

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
               /* in-scene HUD overlays (automata list, minimap, pause …) */
               WorldMenusPlugin,
           ));
    }
}
