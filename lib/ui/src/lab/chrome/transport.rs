//! Transport bar: run/pause, step, reset, metric trace and steps/s selector.
use crate::{
    lab::{Effects, LabData, inspector::sections::format_metric},
    theme::{
        tokens::*,
        widgets::{caps, mono, pill_with_key, segmented},
    },
};
use bevy_egui::egui::{self, Frame, Margin, Pos2, Rect, Sense, Stroke, pos2, vec2};
use engine_core::automata::channel::{
    ChannelAction, ChannelCommand, RATE_PRESETS, Status,
    metrics::{ChannelMetric, Trace},
};

/// Sparkline vertices: oldest at the left, the trace maximum at the top.
pub fn sparkline(trace: &Trace, rect: Rect) -> Vec<Pos2> {
    let Some((lo, hi)) = trace.range() else {
        return Vec::new();
    };
    let n = trace.len();
    if n < 2 {
        return Vec::new();
    }
    trace
        .samples()
        .enumerate()
        .map(|(i, s)| {
            let x = rect.left() + rect.width() * i as f32 / (n - 1) as f32;
            let t = if hi > lo {
                ((s.value - lo) / (hi - lo)) as f32
            } else {
                0.5
            };
            pos2(x, rect.bottom() - t * rect.height())
        })
        .collect()
}

/// Whether `preset` is the highlighted rate for a channel running at `rate_hz`.
pub fn rate_selected(rate_hz: f32, preset: f32) -> bool {
    (rate_hz - preset).abs() < 0.5
}

/// The preset that `rate_selected` matches for `rate_hz`, if any.
pub fn rate_choice(rate_hz: f32) -> Option<f32> {
    RATE_PRESETS
        .into_iter()
        .find(|&p| rate_selected(rate_hz, p))
}

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
                .inner_margin(Margin::symmetric(12, 0)),
        )
        .show(ctx, |ui| {
            let top = ui.max_rect().top();
            let (l, r) = (ui.max_rect().left() - 12.0, ui.max_rect().right() + 12.0);
            ui.painter().hline(l..=r, top, Stroke::new(1.0, LINE));
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
                divider(ui);
                trace(ui, row.and_then(|r| data.metrics.get(r.id)), 230.0);
                divider(ui);
                ui.label(caps("steps/s"));
                let options: Vec<(f32, String)> = RATE_PRESETS
                    .iter()
                    .map(|p| (*p, format!("{p:.0}")))
                    .collect();
                let options: Vec<(f32, &str)> =
                    options.iter().map(|(p, t)| (*p, t.as_str())).collect();
                // NaN equals no preset, so nothing is highlighted off-preset.
                let mut value = row
                    .and_then(|r| rate_choice(r.settings.rate_hz))
                    .unwrap_or(f32::NAN);
                if segmented(ui, &mut value, &options) {
                    send(ChannelAction::SetRate(value));
                }
            });
        });
}

/// Metric label, sparkline filling the space, and the latest value.
fn trace(ui: &mut egui::Ui, metric: Option<&ChannelMetric>, reserve: f32) {
    let Some(metric) = metric else { return };
    ui.label(caps(metric.id.label));
    let value = metric
        .latest
        .map(|s| format_metric(s.value, &metric.id))
        .unwrap_or_default();
    let width = (ui.available_width() - reserve).max(40.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    let points = sparkline(&metric.trace, rect.shrink2(vec2(0.0, 4.0)));
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(1.4, SIGNAL)));
    ui.label(mono(value).size(size::BODY));
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::automata::channel::metrics::MetricSample;

    #[test]
    fn sparkline_spans_the_rect_with_max_at_the_top() {
        let mut trace = Trace::new(10);
        for (t, v) in [(0, 0.0), (1, 5.0), (2, 10.0)] {
            trace.push(MetricSample {
                tick: t,
                epoch: 0,
                value: v,
            });
        }
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
        let points = sparkline(&trace, rect);
        assert_eq!(points.first().unwrap(), &pos2(0.0, 20.0));
        assert_eq!(points.last().unwrap(), &pos2(100.0, 0.0));
    }

    #[test]
    fn flat_or_short_traces() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 20.0));
        let mut trace = Trace::new(10);
        trace.push(MetricSample {
            tick: 0,
            epoch: 0,
            value: 3.0,
        });
        assert!(
            sparkline(&trace, rect).is_empty(),
            "one sample draws nothing"
        );
        trace.push(MetricSample {
            tick: 1,
            epoch: 0,
            value: 3.0,
        });
        assert!(
            sparkline(&trace, rect).iter().all(|p| p.y == 10.0),
            "flat trace sits mid-height"
        );
    }

    #[test]
    fn rate_presets_mark_the_nearest_rate() {
        assert!(rate_selected(10.0, 10.0));
        assert!(
            rate_selected(10.2, 10.0),
            "clamped f32 rates still highlight"
        );
        assert!(!rate_selected(30.0, 10.0));
    }

    #[test]
    fn rate_choice_is_none_off_preset() {
        assert_eq!(rate_choice(12.5), None);
        assert_eq!(rate_choice(10.2), Some(10.0));
        assert_eq!(rate_choice(60.0), Some(60.0));
    }
}
