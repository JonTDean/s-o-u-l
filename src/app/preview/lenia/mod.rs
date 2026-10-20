//! Deterministic launch into the real workbench spawn and control flow.
use bevy::prelude::*;
use engine_core::{events::AutomataCommand, prelude::AppState};
mod verification;

pub(super) fn install(app: &mut App) {
    verification::install(app);
}

pub(super) fn enabled() -> bool {
    std::env::args().any(|argument| argument == "--lenia-demo")
}

pub(super) fn setup(
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
    mut spawn: EventWriter<AutomataCommand>,
) {
    next.set(AppState::InGame);
    spawn.write(AutomataCommand::SeedPattern { id: "lenia".into() });
    super::scene::camera(&mut commands, Vec3::new(128.0, 128.0, 16.0));
}
