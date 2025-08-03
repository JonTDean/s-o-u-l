use bevy::prelude::*;

use crate::components::banners::pause_banner::PauseBannerPlugin;

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct BannersPlugin;

impl Plugin for BannersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PauseBannerPlugin); 
    }
}
