//! Transport bar: run/pause, step, reset, metric trace and steps/s selector.
#[cfg(test)]
mod tests;
mod trace;

pub use trace::{rate_choice, rate_selected, sparkline};

use crate::{
    lab::{Effects, LabData},
    theme::{
        fonts::{SANS, family},
        tokens::*,
        widgets::{caps, pill_with_key, segmented},
    },
};
use bevy_egui::egui::{self, Frame, Margin, Sense, Stroke, vec2};
use engine_core::automata::channel::{
    ChannelAction, ChannelCommand, RATE_PRESETS, Status, metrics::ChannelMetric,
};

/// Horizontal inner margin of the bar.
const MARGIN: f32 = 12.0;
fn divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 22.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, LINE);
}

/// Draw the bottom transport bar for the selected channel.
pub fn show(ctx: &egui::Context, data: &LabData, fx: &mut Effects) {
    egui::TopBottomPanel::bottom("lab.transport")
        .exact_height(44.0)
        .frame(
            Frame::new()
                .fill(PANEL)
                .stroke(Stroke::NONE)
                .inner_margin(Margin::symmetric(MARGIN as i8, 0)),
        )
        .show(ctx, |ui| {
            let top = ui.max_rect().top();
            let edge = ui.max_rect().expand2(vec2(MARGIN, 0.0));
            ui.painter()
                .hline(edge.x_range(), top, Stroke::new(1.0, LINE));
            ui.horizontal_centered(|ui| {
                let row = data.channels.selected_row();
                let status = row.map(|r| r.status);
                let mut send = |action| {
                    if let Some(r) = row {
                        fx.commands.push(ChannelCommand { id: r.id, action });
                    }
                };
                let running = status == Some(Status::Running);
                let runnable = matches!(status, Some(Status::Running | Status::Paused));
                let label = if running { "‖ Pause" } else { "▶ Run" };
                if pill_with_key(ui, label, "Space", true, runnable).clicked() {
                    send(ChannelAction::Pause(running));
                }
                let paused = status == Some(Status::Paused);
                if pill_with_key(ui, "⏭ Step", ".", false, paused).clicked() {
                    send(ChannelAction::Step);
                }
                if pill_with_key(ui, "↺ Reset", "R", false, row.is_some()).clicked() {
                    send(ChannelAction::Reset);
                }
                let metric = row.and_then(|r| data.metrics.get(r.id));
                let rate = row.map(|r| r.settings.rate_hz);
                if let Some(preset) = tail(ui, metric, rate) {
                    send(ChannelAction::SetRate(preset));
                }
            });
        });
}

/// Width of one row of all rate presets in `segmented`.
fn selector_width(ui: &egui::Ui, labels: &[&str]) -> f32 {
    let font = egui::FontId::new(size::BODY, family(SANS));
    let widest = labels
        .iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap((*l).into(), font.clone(), INK)
                .size()
                .x
        })
        .fold(0.0, f32::max);
    let (_, per) = (labels.len(), widest + 12.0 + 2.0);
    labels.len() as f32 * per + 2.0
}

/// Trace and rate selector; returns a newly chosen rate preset. The selector
/// is laid out first, flush right on one row; the trace takes what remains.
fn tail(ui: &mut egui::Ui, metric: Option<&ChannelMetric>, rate: Option<f32>) -> Option<f32> {
    divider(ui);
    let texts: Vec<String> = RATE_PRESETS.iter().map(|p| format!("{p:.0}")).collect();
    let options: Vec<(f32, &str)> = RATE_PRESETS
        .iter()
        .zip(&texts)
        .map(|(p, t)| (*p, t.as_str()))
        .collect();
    let width = selector_width(ui, &texts.iter().map(String::as_str).collect::<Vec<_>>());
    // NaN equals no preset, so nothing is highlighted off-preset.
    let mut value = rate.and_then(rate_choice).unwrap_or(f32::NAN);
    let mut changed = false;
    let centered = egui::Layout::right_to_left(egui::Align::Center);
    ui.with_layout(centered, |ui| {
        ui.allocate_ui_with_layout(vec2(width, 28.0), centered, |ui| {
            changed = segmented(ui, &mut value, &options);
        });
        ui.label(caps("steps/s"));
        divider(ui);
        let left = egui::Layout::left_to_right(egui::Align::Center);
        let rest = vec2(ui.available_width().max(0.0), 26.0);
        ui.allocate_ui_with_layout(rest, left, |ui| trace::trace(ui, metric));
    });
    changed.then_some(value)
}
