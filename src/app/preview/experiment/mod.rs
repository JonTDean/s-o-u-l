//! Native experiment workflow qualification through production project controls.
use bevy::prelude::*;
use engine_core::{events::AutomataCommand, prelude::AppState};
mod authoring;
mod capture;
mod checkpoint;
mod flow;
mod worlds;
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
    std::env::args().any(|a| a == "--experiment-demo" || a == "--rules-demo")
}
pub(super) fn install(app: &mut App) {
    worlds::install(app);
    authoring::install(app);
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
    mut files: ResMut<ui::lab::chrome::drawers::Files>,
) {
    next.set(AppState::InGame);
    if std::env::args().any(|a| a == "--rules-demo") {
        files.load = Some(authoring::rules::demo());
        super::scene::camera(&mut commands, Vec3::new(128.0, 128.0, 16.0));
        return;
    }
    events.write(AutomataCommand::SeedPattern { id: "lenia".into() });
    if worlds::enabled() {
        events.write(AutomataCommand::SeedPattern { id: "lenia".into() });
    }
    for id in ["life:conway", "wolfram:rule30"] {
        events.write(AutomataCommand::SeedPattern { id: id.into() });
    }
    super::scene::camera(&mut commands, Vec3::new(128.0, 128.0, 16.0));
}
