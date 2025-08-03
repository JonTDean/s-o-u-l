//! overlays/debug_stats.rs
//!
//! Collapsible “Debug Stats” panel shown in-game (F4).
//!
//! Displays:
//! • Instant FPS + frame time
//! • Simulation steps this frame
//! • GPU time per sim-step (when the `engine-gpu` plugin is active)

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::prelude::AppState;

#[cfg(feature = "gpu-compute")]
use engine_gpu::plugin::StepsThisFrame;

/// Toggleable overlay; press **F4** to open / close.
pub fn debug_stats_panel(
    mut egui_ctx:   EguiContexts<'_, '_>,
    keys:           Res<ButtonInput<KeyCode>>,
    diagnostics:    Res<DiagnosticsStore>,

    #[cfg(feature = "gpu-compute")]
    step_info: Option<Res<StepsThisFrame>>,

    mut open: Local<bool>,
) {
    if keys.just_pressed(KeyCode::F4) {
        *open = !*open;
    }
    if !*open {
        return;
    }

    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    // ── instantaneous values (update every frame) ──────────────────
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.value())                    // ← was  smoothed()
        .unwrap_or_default();

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.value())                    // ← was  smoothed()
        .unwrap_or_default() * 1_000.0;

    egui::Window::new("Debug Stats")
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!("FPS:   {:.1}", fps));
            ui.label(format!("Frame: {:.2} ms", frame_ms));

            #[cfg(feature = "gpu-compute")]
            if let Some(info) = step_info {
                ui.label(format!("Sim steps: {}", info.steps));
                ui.label(format!("GPU step:  {:.2} ms", info.gpu_time_ms));
            }
        });
}
