//! Reproducible 3D voxel preview with an optional window-only screenshot.
use bevy::prelude::*;
mod automata;
mod boot;
mod capture;
mod experiment;
mod lenia;
mod scene;

pub(super) fn enabled() -> bool {
    std::env::args().any(|argument| argument == "--voxel-demo")
        || lenia::enabled()
        || automata::enabled()
        || experiment::enabled()
}

pub(super) fn install(app: &mut App) {
    boot::install(app);
    if experiment::enabled() {
        experiment::install(app);
        app.add_systems(PostStartup, experiment::setup);
    } else if automata::enabled() {
        automata::install(app);
        app.add_systems(PostStartup, automata::setup);
    } else if lenia::enabled() {
        lenia::install(app);
        app.add_systems(PostStartup, lenia::setup);
    } else {
        app.add_systems(PostStartup, scene::setup);
    }
    app.add_systems(Update, scene::isolate_preview_camera)
        .add_systems(Update, capture::capture_preview);
}
