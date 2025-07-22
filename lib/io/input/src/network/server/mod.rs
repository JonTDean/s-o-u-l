use bevy::prelude::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, _app: &mut App) {
        info!("Server networking enabled (stub)");
    }
}