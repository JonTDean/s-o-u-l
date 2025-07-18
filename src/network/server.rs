use bevy::prelude::*;

/// Placeholder server-side networking plugin.
pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, _app: &mut App) {
        info!("Server networking enabled (stub)");
    }
}
