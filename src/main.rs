mod state_engine;
mod app_state;
mod ui;
mod tests;

use bevy::prelude::*;
use ui::components::menus::MenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "S.O.U.L. - Swarm Orchestrator for Autonomous Learners".into(), ..default() }),
            ..default()
        }))
        .add_plugins(MenuPlugin)           // ← menus load first
        .run();
}
