//! Native acceptance journey for specimens, guides, voxel entities and published Orbium.
use bevy::prelude::*;
use engine_core::automata::{
    channel::{ChannelId, Channels},
    lenia::LeniaSimulation,
};
use models::workbench::{
    plugin::Workbench,
    project::{self, library},
};
use ui::lab::{
    chrome::drawers::{Comparison, Files, Request},
    state::LabState,
};
mod cpu;
mod flow;
mod gpu;
mod rules;
#[derive(Resource)]
struct Verification {
    output: std::path::PathBuf,
    phase: u8,
    frames: u32,
    wait: u32,
    captured: std::sync::Arc<std::sync::atomic::AtomicBool>,
    expected: Option<serde_json::Value>,
}
pub(super) fn install(app: &mut App) {
    rules::install(app);
    let Some(output) = std::env::var_os("SOUL_AUTHORING_VERIFY") else {
        return;
    };
    let output = std::path::PathBuf::from(output);
    std::fs::create_dir(&output).expect("fresh authoring verification directory");
    app.insert_resource(Verification {
        output,
        phase: 0,
        frames: 0,
        wait: 0,
        captured: default(),
        expected: None,
    })
    .add_systems(Last, flow::update);
}
fn capture(world: &mut World, v: &mut Verification, name: &str) {
    use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
    v.captured
        .store(false, std::sync::atomic::Ordering::Release);
    let signal = v.captured.clone();
    let mut save = save_to_disk(v.output.join(name));
    world.spawn(Screenshot::primary_window()).observe(
        move |trigger: Trigger<ScreenshotCaptured>| {
            save(trigger);
            signal.store(true, std::sync::atomic::Ordering::Release);
        },
    );
}
