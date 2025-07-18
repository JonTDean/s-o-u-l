mod intelligence_engine;
mod state;
mod ui;
mod tests;
mod dev_utils;

use bevy::prelude::*;
use state::StatePlugin;
use ui::components::{
    file_io::FileIoPlugin,
    menus::main_menu::MainMenuPlugin,
};
use intelligence_engine::renderer::Renderer2DPlugin;
use dev_utils::tools::quit::quit_on_esc;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "S.O.U.L. – Swarm Orchestrator for Autonomous Learners".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, quit_on_esc)
        .init_state::<state::AppState>()
        .add_plugins((
            StatePlugin,
            MainMenuPlugin,
            Renderer2DPlugin,
            FileIoPlugin, 
        ))
        .run();
}
