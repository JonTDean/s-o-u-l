//! Tufte-style margin sections: label column, content hanging beside it.
use super::readout::dim;
use crate::theme::tokens::*;
use bevy_egui::egui::{Align, Layout, Ui, vec2};

/// Width of the right-aligned margin label column.
pub const MARGIN_WIDTH: f32 = 56.0;

/// Draw `label` in the margin (amber when `active`) and `body` beside it.
pub fn margin_section(ui: &mut Ui, label: &str, active: bool, body: impl FnOnce(&mut Ui)) {
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            vec2(MARGIN_WIDTH, 14.0),
            Layout::right_to_left(Align::Min),
            |ui| {
                ui.label(dim(label).color(if active { SIGNAL } else { INK_DIM }));
            },
        );
        ui.vertical(body);
    });
    ui.add_space(10.0);
}

/// Small dim note; `warn` uses the signal colour.
pub fn note(ui: &mut Ui, text: &str, warn: bool) {
    ui.label(
        dim(text)
            .size(size::BODY - 1.0)
            .color(if warn { SIGNAL } else { INK_DIM }),
    );
}
