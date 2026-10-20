//! Native verification of authored worlds, quantitative comparison and continuous Lenia.
use bevy::prelude::*;
use engine_core::automata::{
    channel::ChannelId,
    lenia::{LeniaSimulation, SavedLenia},
};
use models::workbench::plugin::Workbench;
mod checkpoint;
mod flow;
mod scenario;
#[derive(Resource)]
struct Verification {
    output: std::path::PathBuf,
    phase: u8,
    frames: u32,
    captured: std::sync::Arc<std::sync::atomic::AtomicBool>,
    saved: Vec<SavedLenia>,
    old_owner: Option<Entity>,
}
pub(super) fn enabled() -> bool {
    std::env::var_os("SOUL_WORLDS_VERIFY").is_some()
}
pub(super) fn install(app: &mut App) {
    let Some(output) = std::env::var_os("SOUL_WORLDS_VERIFY") else {
        return;
    };
    let output = std::path::PathBuf::from(output);
    std::fs::create_dir(&output).expect("choose a fresh worlds verification directory");
    app.insert_resource(Verification {
        output,
        phase: 0,
        frames: 0,
        captured: Default::default(),
        saved: Vec::new(),
        old_owner: None,
    })
    .add_systems(Last, flow::update);
}
