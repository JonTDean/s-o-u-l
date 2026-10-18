//! Reproducible 3D voxel preview with an optional window-only screenshot.
use bevy::prelude::*;
mod capture;
mod lenia;
mod scene;

pub(super) fn enabled() -> bool {
    std::env::args().any(|argument| argument == "--voxel-demo") || lenia::enabled()
}

pub(super) fn install(app: &mut App) {
    if lenia::enabled() {
        app.add_systems(PostStartup, lenia::setup);
    } else {
        app.add_systems(PostStartup, scene::setup);
    }
    app.add_systems(Update, scene::isolate_preview_camera)
        .add_systems(Update, capture::capture_preview);
}
