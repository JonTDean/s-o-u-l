use bevy::app::{App, Plugin};
use tooling::debugging::floor::DebugFloorPlugin;
use crate::render::materials::debug::debug_grid::DebugGridPlugin;

pub struct DebugMaterialsPlugin;

impl Plugin for DebugMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugFloorPlugin);
        app.add_plugins(DebugGridPlugin);
    }
}