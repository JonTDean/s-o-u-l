//! Pills, status lamps, thin sliders and plain radio choices.
use super::readout::{dim, mono, plain};
use crate::theme::{
    fonts::{SANS_SEMIBOLD, family},
    tokens::*,
};
use bevy_egui::egui::{self, Color32, CornerRadius, Response, RichText, Sense, Ui, vec2};
use engine_core::automata::channel::Status;
use std::ops::RangeInclusive;

/// Pill fill: amber only for an enabled primary action.
pub fn pill_fill(primary: bool, enabled: bool) -> Color32 {
    if primary && enabled { SIGNAL } else { RAISED }
}

/// Rounded, borderless action button.
pub fn pill(ui: &mut Ui, label: &str, primary: bool, enabled: bool) -> Response {
    let text = if primary && enabled {
        RichText::new(label).family(family(SANS_SEMIBOLD)).color(BG)
    } else {
        RichText::new(label).color(if enabled { INK } else { TRACK })
    };
    let button = egui::Button::new(text)
        .fill(pill_fill(primary, enabled))
        .corner_radius(CornerRadius::same(10));
    ui.add_enabled(enabled, button)
}

/// Lamp colour per lifecycle state.
pub fn lamp_colour(status: Status) -> Color32 {
    match status {
        Status::Running => OK,
        Status::Paused => SIGNAL,
        Status::Halted => FAULT,
    }
}

/// 7 px status lamp; running lamps carry a soft glow.
pub fn lamp(ui: &mut Ui, status: Status) {
    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
    let colour = lamp_colour(status);
    if status == Status::Running {
        ui.painter()
            .circle_filled(rect.center(), 5.0, colour.gamma_multiply(0.25));
    }
    ui.painter().circle_filled(rect.center(), 3.5, colour);
}

/// Labelled thin slider with a mono value; returns whether the value changed.
pub fn slider(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    log: bool,
) -> bool {
    ui.horizontal(|ui| {
        ui.label(dim(label));
        let changed = ui
            .add(
                egui::Slider::new(value, range)
                    .show_value(false)
                    .trailing_fill(true)
                    .logarithmic(log)
                    .handle_shape(egui::style::HandleShape::Circle),
            )
            .changed();
        ui.label(mono(format!("{value:.3}")));
        changed
    })
    .inner
}

/// Plain radio row (`●` selected, `○` not); returns whether the choice changed.
pub fn choice<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        for (value, label) in options {
            let on = *current == *value;
            let text = plain(format!("{} {label}", if on { "●" } else { "○" })).color(if on {
                SIGNAL
            } else {
                INK_DIM
            });
            if ui
                .add(egui::Label::new(text).sense(Sense::click()))
                .clicked()
                && !on
            {
                *current = *value;
                changed = true;
            }
        }
    });
    changed
}
