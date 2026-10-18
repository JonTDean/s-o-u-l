//! Optional capture of the application window for rendering verification.
use bevy::{
    diagnostic::FrameCount,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

pub(super) fn capture_preview(
    mut commands: Commands,
    frames: Res<FrameCount>,
    status: Option<Res<engine_gpu::VoxelRenderStatus>>,
    mut captured: Local<bool>,
) {
    if *captured || frames.0 < 180 || !status.is_some_and(|status| status.has_recorded_draw()) {
        return;
    }
    let Ok(path) = std::env::var("SOUL_SCREENSHOT") else {
        return;
    };
    *captured = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
