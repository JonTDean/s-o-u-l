//! Labelled slider with a boxed mono value.
use crate::theme::{
    tokens::*,
    widgets::readout::{mono, plain},
};
use bevy_egui::egui::{self, Align, Frame, Layout, Margin, Stroke, Ui, vec2};
use std::ops::RangeInclusive;

const LABEL_WIDTH: f32 = 72.0;
const VALUE_WIDTH: f32 = 56.0;

/// Boxed, right-aligned mono value.
pub fn value_box(ui: &mut Ui, text: &str) {
    ui.allocate_ui_with_layout(
        vec2(VALUE_WIDTH, ui.spacing().interact_size.y),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.set_width(VALUE_WIDTH);
            Frame::new()
                .fill(BG)
                .stroke(Stroke::new(1.0, LINE))
                .corner_radius(4.0)
                .inner_margin(Margin::symmetric(5, 1))
                .show(ui, |ui| {
                    ui.set_width(VALUE_WIDTH - 10.0 - 2.0);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(mono(text));
                    });
                });
        },
    );
}

/// Labelled slider with a value box; returns whether the value changed.
pub fn slider(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    log: bool,
) -> bool {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            vec2(LABEL_WIDTH, ui.spacing().interact_size.y),
            Layout::left_to_right(Align::Center),
            |ui| ui.label(plain(label).color(INK)),
        );
        let gap = ui.spacing().item_spacing.x;
        ui.spacing_mut().slider_width = (ui.available_width() - VALUE_WIDTH - gap).max(40.0);
        let changed = ui
            .add(
                egui::Slider::new(value, range)
                    .show_value(false)
                    .trailing_fill(true)
                    .logarithmic(log)
                    .handle_shape(egui::style::HandleShape::Circle),
            )
            .changed();
        value_box(ui, &format!("{value:.3}"));
        changed
    })
    .inner
}
