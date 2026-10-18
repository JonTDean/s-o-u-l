//! State shared between application and render worlds.
use bevy::prelude::*;

/// Handles for the global voxel-atlas textures.
#[derive(Resource, Clone)]
pub struct GlobalVoxelAtlas {
    /// 3-D texture storing automaton state (R8Uint).
    pub atlas: Handle<Image>,
    /// 2-D signalling texture (same X/Y footprint as one atlas layer).
    pub signal: Handle<Image>,
}

/// Toggles even/odd frames for ping-pong resources.
#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

/// Per-frame simulation statistics copied into the render world.
#[derive(Resource, Default, Clone, Copy)]
pub struct StepsThisFrame {
    /// Number of simulation ticks executed *this* frame.
    pub steps: u32,
    /// Reserved GPU timestamp result in milliseconds; zero until timing is implemented.
    pub gpu_time_ms: f32,
}

/// Debug uploads queued across the extraction/prepare boundary until the image exists.
#[cfg(feature = "gpu-debug")]
#[derive(Resource, Default)]
pub(crate) struct PendingDebugSeeds(pub Vec<engine_core::events::DebugSeedSquare>);

/// Shared readiness signal for consumers that need a completed draw recording.
#[derive(Resource, Clone, Default)]
pub struct VoxelRenderStatus {
    draw_recorded: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl VoxelRenderStatus {
    /// Whether a voxel draw has been recorded with ready images and pipelines.
    pub fn has_recorded_draw(&self) -> bool {
        self.draw_recorded
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn mark_draw_recorded(&self) {
        self.draw_recorded
            .store(true, std::sync::atomic::Ordering::Release);
    }
}
