use bevy::app::{App, Plugin};

use crate::render::materials::debug::plugin::DebugMaterialsPlugin;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugMaterialsPlugin);
    }
}