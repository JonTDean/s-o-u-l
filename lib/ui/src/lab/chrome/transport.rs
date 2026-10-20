//! Transport bar: run/pause, step, reset, metric trace and steps/s selector.
use crate::{
    lab::{Effects, LabData},
    theme::{
        fonts::{SERIF_ITALIC, family},
        tokens::*,
        widgets::{mono, pill, serif_italic},
    },
};
use bevy_egui::egui::{self, Frame, Margin, Pos2, Rect, Sense, Stroke, pos2, vec2};
use engine_core::automata::channel::{
    ChannelAction, ChannelCommand, RATE_PRESETS, Status, metrics::Trace,
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

/// Draw the bottom transport bar for the selected channel.
pub fn show(ctx: &egui::Context, data: &LabData, fx: &mut Effects) {
    egui::TopBottomPanel::bottom("lab.transport")
        .exact_height(40.0)
        .frame(
            Frame::new()
                .fill(PANEL)
                .inner_margin(Margin::symmetric(12, 0)),
        )
        .show(ctx, |ui| {
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
                if pill(ui, label, true, runnable).clicked() {
                    send(ChannelAction::Pause(running));
                }
                if pill(ui, "Step", false, status == Some(Status::Paused)).clicked() {
                    send(ChannelAction::Step);
                }
                if pill(ui, "Reset", false, row.is_some()).clicked() {
                    send(ChannelAction::Reset);
                }
                let width = (ui.available_width() - 150.0).max(40.0);
                let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
                if let Some(metric) = row.and_then(|r| data.metrics.get(r.id)) {
                    let font = egui::FontId::new(size::LABEL, family(SERIF_ITALIC));
                    let painter = ui.painter();
                    painter.text(
                        rect.left_top(),
                        egui::Align2::LEFT_TOP,
                        metric.id.label,
                        font,
                        INK_DIM,
                    );
                    let points = sparkline(&metric.trace, rect.shrink2(vec2(0.0, 4.0)));
                    painter.add(egui::Shape::line(points, Stroke::new(1.2, SIGNAL)));
                }
                ui.label(serif_italic("steps/s"));
                for preset in RATE_PRESETS {
                    let on = row.is_some_and(|r| rate_selected(r.settings.rate_hz, preset));
                    let text =
                        mono(format!("{preset:.0}")).color(if on { SIGNAL } else { INK_DIM });
                    if ui
                        .add(egui::Label::new(text).sense(Sense::click()))
                        .clicked()
                    {
                        send(ChannelAction::SetRate(preset));
                    }
                }
            });
        });
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
}
