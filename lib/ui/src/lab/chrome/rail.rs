//! 32 px icon rail: library (painted book spines), compare (≡), project (§) and settings (⁂).
use crate::{
    lab::{
        Effects, LabData,
        state::{Drawer, LabMode, LabState},
    },
    theme::{
        fonts::{SANS, family},
        tokens::*,
        widgets::{note, plain},
    },
};
use bevy_egui::egui::{self, Frame, Margin, RichText, Sense, Stroke, Ui, vec2};
use engine_core::automata::channel::Backend;

/// Draw the rail and, when open, the settings popover.
pub fn show(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    egui::SidePanel::left("lab.rail")
        .exact_width(32.0)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(PANEL)
                .inner_margin(Margin::symmetric(0, 10)),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                library_icon(ui, lab);
                glyph(ui, lab, "≡", "Compare sessions", Drawer::Compare);
                glyph(ui, lab, "§", "Project save and load", Drawer::Project);
                let gear = RichText::new("⁂")
                    .family(family(SANS))
                    .size(16.0)
                    .color(if lab.settings_open { SIGNAL } else { INK_DIM });
                let settings = ui.add(egui::Label::new(gear).sense(Sense::click()));
                if settings.on_hover_text("Settings").clicked() {
                    lab.settings_open = !lab.settings_open;
                }
            });
        });
    if lab.settings_open {
        settings(ctx, lab, data, fx);
    }
}

/// Three painted book spines; Plex has no `❦`.
fn library_icon(ui: &mut Ui, lab: &mut LabState) {
    let (rect, library) = ui.allocate_exact_size(vec2(32.0, 20.0), Sense::click());
    let colour = icon_colour(lab, Drawer::Library);
    for (dx, h) in [(-5.0, 12.0), (0.0, 14.0), (5.0, 11.0)] {
        let x = rect.center().x + dx;
        let bottom = rect.bottom() - 3.0;
        ui.painter().line_segment(
            [egui::pos2(x, bottom), egui::pos2(x, bottom - h)],
            Stroke::new(2.5, colour),
        );
    }
    if library.on_hover_text("Library (L)").clicked() {
        lab.toggle_drawer(Drawer::Library);
    }
}

fn glyph(ui: &mut Ui, lab: &mut LabState, symbol: &str, hint: &str, drawer: Drawer) {
    let text = RichText::new(symbol)
        .family(family(SANS))
        .size(16.0)
        .color(icon_colour(lab, drawer));
    let response = ui.add(egui::Label::new(text).sense(Sense::click()));
    if response.on_hover_text(hint).clicked() {
        lab.toggle_drawer(drawer);
    }
}

fn icon_colour(lab: &LabState, drawer: Drawer) -> egui::Color32 {
    if lab.drawer == Some(drawer) {
        SIGNAL
    } else {
        INK_DIM
    }
}

fn settings(ctx: &egui::Context, lab: &mut LabState, data: &LabData, fx: &mut Effects) {
    egui::Area::new("lab.settings".into())
        .fixed_pos(egui::pos2(36.0, 44.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            Frame::new()
                .fill(PANEL)
                .inner_margin(12)
                .corner_radius(6)
                .show(ui, |ui| {
                    ui.label(plain("Settings").size(size::TITLE));
                    if ui.button("Presenter mode (P)").clicked() {
                        lab.mode = LabMode::Presenter;
                        lab.settings_open = false;
                    }
                    let gpu = data
                        .channels
                        .selected_row()
                        .is_some_and(|r| r.backend == Backend::Gpu);
                    let mark = egui::Button::new("Mark centre of selected GPU specimen");
                    if ui.add_enabled(gpu, mark).clicked() {
                        fx.mark_centre = true;
                    }
                    note(ui, "Debug overlay: F3", false);
                });
        });
}
