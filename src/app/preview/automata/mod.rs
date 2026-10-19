//! Launch into the multi-family workbench with a visible elementary history.
use bevy::prelude::*;
use engine_core::{events::AutomataCommand, prelude::AppState};
use models::workbench::{catalog::Kind, plugin::Workbench};

mod capture;
mod flow;
mod verification;

pub(super) fn enabled() -> bool {
    std::env::args().any(|argument| argument == "--automata-demo")
}

pub(super) fn install(app: &mut App) {
    verification::install(app);
    app.add_systems(Update, prepare_example);
}

pub(super) fn setup(
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
    mut events: EventWriter<AutomataCommand>,
) {
    next.set(AppState::InGame);
    events.write(AutomataCommand::SeedPattern {
        id: Kind::Rule30.id().into(),
    });
    super::scene::camera(
        &mut commands,
        Vec3::ZERO,
        "S.O.U.L. / Automata lab\nChoose a family, then resume, step, or reset its session.",
    );
}

fn prepare_example(mut bench: ResMut<Workbench>, mut ready: Local<bool>) {
    if *ready {
        return;
    }
    let Some(session) = bench.sessions.first_mut() else {
        return;
    };
    for _ in 0..32 {
        session.single_step();
    }
    *ready = true;
}
