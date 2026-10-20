//! Output-layer root: theme, lab shell, menus and debug overlays.
use crate::{components::plugin::ComponentsPlugin, lab::LabPlugin, theme::ThemePlugin};
use bevy::prelude::*;

/// Adds every output feature.
pub struct OutputPlugin;

impl Plugin for OutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThemePlugin, ComponentsPlugin, LabPlugin));
    }
}
