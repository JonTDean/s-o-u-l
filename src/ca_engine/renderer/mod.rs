use bevy::prelude::*;

pub mod camera;
pub mod input;
pub mod draw;

pub struct Renderer2DPlugin;

impl Plugin for Renderer2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera::setup_camera)
           .add_systems(Update, input::handle_mouse_clicks)
           .add_systems(PostUpdate, draw::paint_dense_board);
    }
}