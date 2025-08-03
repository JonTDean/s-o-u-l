use bevy::prelude::*;

use crate::components::{banners::plugin::BannersPlugin, debug::plugin::DebugComponentsPlugin};

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BannersPlugin)
           .add_plugins(DebugComponentsPlugin); 
    }
}
