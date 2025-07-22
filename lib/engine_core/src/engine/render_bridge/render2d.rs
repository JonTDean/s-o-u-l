use bevy::prelude::*;

pub struct Renderer2DPlugin;
impl Plugin for Renderer2DPlugin {
    fn build(&self, _app: &mut App) {
        info!("Renderer2DPlugin initialized (stub)");
        // In a full implementation, this would setup camera, sprites, and rendering systems.
    }
}
