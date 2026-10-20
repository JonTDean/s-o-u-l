//! Native experiment workflow qualification through production project controls.
use bevy::prelude::*;
use engine_core::{events::AutomataCommand, prelude::AppState};
mod capture;
mod checkpoint;
mod flow;
#[derive(Resource)]
pub(super) struct Verification {
    output: std::path::PathBuf,
    phase: u8,
    frames: u32,
    owner: Option<Entity>,
    expected: Vec<u8>,
    captured: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
pub(super) fn enabled() -> bool {
    std::env::args().any(|a| a == "--experiment-demo")
}
pub(super) fn install(app: &mut App) {
    if let Some(output) = std::env::var_os("SOUL_EXPERIMENT_VERIFY") {
        let output = std::path::PathBuf::from(output);
        std::fs::create_dir_all(&output).unwrap();
        assert!(
            !output.join("SUCCESS").exists(),
            "Choose a fresh qualification directory"
        );
        app.insert_resource(Verification {
            output,
            phase: 0,
            frames: 0,
            owner: None,
            expected: Vec::new(),
            captured: Default::default(),
        })
        .add_systems(Last, flow::update);
    }
}
pub(super) fn setup(
    mut commands: Commands,
    mut next: ResMut<NextState<AppState>>,
    mut events: EventWriter<AutomataCommand>,
) {
    next.set(AppState::InGame);
    events.write(AutomataCommand::SeedPattern { id: "lenia".into() });
    for id in ["life:conway", "wolfram:rule30"] {
        events.write(AutomataCommand::SeedPattern { id: id.into() });
    }
    super::scene::camera(&mut commands, Vec3::new(128.0, 128.0, 16.0));
}
