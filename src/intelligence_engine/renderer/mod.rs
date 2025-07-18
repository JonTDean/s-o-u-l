use bevy::prelude::*;
use crate::state::AppState;
use self::{camera::setup_camera, draw::paint_dense_board, input::handle_mouse_clicks};

pub mod camera;
pub mod input;
pub mod draw;

pub struct Renderer2DPlugin;

impl Plugin for Renderer2DPlugin {
    fn build(&self, app: &mut App) {
        app
            // camera once, when we ENTER the game state
            .add_systems(OnEnter(AppState::InGame), setup_camera)
            // mouse input while we ARE in game
            .add_systems(
                Update,
                handle_mouse_clicks.run_if(in_state(AppState::InGame)),
            )
            // redraw right after the logic, still only in game
            .add_systems(
                PostUpdate,
                paint_dense_board.run_if(in_state(AppState::InGame)),
            );
    }
}
