use bevy::prelude::*;

use crate::styles::fade::FadePlugin;

/// Bundles every in-game egui overlay plus debug gizmos.
pub struct StylesPlugin;

impl Plugin for StylesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                /* global fade-in / fade-out transitions                   */
                FadePlugin,
           ));
    }
}
