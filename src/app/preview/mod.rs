//! Reproducible 3D voxel preview with an optional window-only screenshot.
use bevy::prelude::*;
mod capture;
mod scene;

pub(super) fn enabled() -> bool {
    std::env::args().any(|argument| argument == "--voxel-demo")
}

pub(super) fn install(app: &mut App) {
    app.add_systems(PostStartup, scene::setup)
        .add_systems(Update, scene::isolate_preview_camera)
        .add_systems(Update, capture::capture_preview);
}
