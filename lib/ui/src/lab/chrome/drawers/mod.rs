//! Rail drawers: one panel slides over the stage beside the rail, dimming what is behind it.
use crate::{
    lab::{
        Effects, LabData,
        state::{Drawer, LabState},
    },
    theme::tokens::*,
};
use bevy_egui::egui::{self, Frame, Sense};
pub mod compare;
pub mod library;
pub mod project;

/// Drawer width in logical points.
pub const WIDTH: f32 = 300.0;

/// Draw the dimming layer and the open drawer, if any.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    let Some(drawer) = lab.drawer else { return };
    let screen = ctx.screen_rect();
    let dim = egui::Rect::from_min_max(egui::pos2(32.0, 0.0), screen.max);
    egui::Area::new("lab.dim".into())
        .fixed_pos(dim.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let response = ui.allocate_rect(dim, Sense::click());
            ui.painter().rect_filled(dim, 0.0, BG.gamma_multiply(0.55));
            if response.clicked() {
                lab.drawer = None;
            }
        });
    egui::Area::new("lab.drawer".into())
        .fixed_pos(egui::pos2(32.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let margin = egui::Margin {
                left: 4,
                right: 14,
                top: 14,
                bottom: 14,
            };
            Frame::new()
                .fill(PANEL)
                .inner_margin(margin)
                .show(ui, |ui| {
                    ui.set_width(WIDTH);
                    ui.set_min_height(screen.height() - 28.0);
                    egui::ScrollArea::vertical().show(ui, |ui| match drawer {
                        Drawer::Library => library::body(ui, lab, data, fx),
                        Drawer::Compare => compare::body(ui, lab, data, fx),
                        Drawer::Project => project::body(ui, lab, data, fx),
                    });
                });
        });
}
