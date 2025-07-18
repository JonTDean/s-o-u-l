use bevy::prelude::*;

/// Placeholder client-side networking plugin.
pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, _app: &mut App) {
        info!("Client networking enabled (stub)");
    }
}
