//! World-camera HUD overlay (zoom + coordinate read-out).
//!
//! Anchored to the bottom-right corner via **egui::Area**.  Displays:
//!
//! * Current **zoom** as a signed percentage relative to
//!   [`ZoomInfo::base`].
//! * **World-space camera centre** – X and Y rounded to whole units.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_common::controls::camera::{WorldCamera, ZoomInfo};

pub mod zoom_overlay;

/// Renders the overlay every frame (when in-game).
pub fn camera_overlay(
    mut egui_ctx: EguiContexts,
    zoom:        Res<ZoomInfo>,
    cam_q:       Query<&Transform, With<WorldCamera>>,
) {
    let Ok(ctx)        = egui_ctx.ctx_mut()        else { return };
    let Some(cam_tf)   = cam_q.iter().next()       else { return };

    let pos  = cam_tf.translation.truncate();
    let pct = if zoom.base > 0.0 {
        (zoom.current / zoom.base - 1.0) * 100.0
    } else { 0.0 };

    egui::Area::new("camera_overlay".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
        .show(ctx, |ui| {
            ui.label(format!("Zoom: {pct:+.0}%  |  Pos: ({:.0}, {:.0})", pos.x, pos.y));
        });
}
