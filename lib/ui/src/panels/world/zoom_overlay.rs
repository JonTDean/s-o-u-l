use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine::renderer::components::ZoomInfo;

pub fn zoom_overlay(mut egui_ctx: EguiContexts, zoom: Res<ZoomInfo>) {
    let ctx = egui_ctx.ctx_mut().unwrap();
    egui::Area::new("zoom_overlay".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .show(ctx, |ui| {
            let pct = (zoom.current / zoom.base - 1.0) * 100.0;
            ui.label(format!("Zoom: {pct:+.0}%"));
        });
}
