use bevy::prelude::*;

pub mod dev_utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Demo".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        // pass the function *without* parentheses
        .add_systems(Update, dev_utils::tools::quit::quit_on_esc)
        .run();
}
