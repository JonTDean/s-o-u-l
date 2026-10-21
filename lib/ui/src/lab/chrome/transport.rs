//! Transport bar: run/pause, step, reset, metric trace and steps/s selector.
use crate::{
    lab::{Effects, LabData, inspector::sections::format_metric},
    theme::{
        fonts::{MONO_MEDIUM, SANS, family},
        tokens::*,
        widgets::{caps, mono, pill_with_key, segmented},
    },
};
use bevy_egui::egui::{self, Frame, Margin, Pos2, Rect, Sense, Stroke, pos2, vec2};
use engine_core::automata::channel::{
    ChannelAction, ChannelCommand, RATE_PRESETS, Status,
    metrics::{ChannelMetric, Trace},
};

/// Horizontal inner margin of the bar.
const MARGIN: f32 = 12.0;

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
        ui.allocate_ui_with_layout(rest, left, |ui| trace(ui, metric));
    });
    changed.then_some(value)
}

/// Metric label, sparkline filling the space, and the latest value.
fn trace(ui: &mut egui::Ui, metric: Option<&ChannelMetric>) {
    let Some(metric) = metric else { return };
    ui.label(caps(metric.id.label));
    let value = metric
        .latest
        .map(|s| format_metric(s.value, &metric.id))
        .unwrap_or_default();
    let text = mono(value).family(family(MONO_MEDIUM)).size(size::BODY);
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            text.text().into(),
            egui::FontId::new(size::BODY, family(MONO_MEDIUM)),
            SIGNAL,
        )
    });
    let width =
        (ui.available_width() - galley.size().x - 2.0 * ui.spacing().item_spacing.x).max(0.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 26.0), Sense::hover());
    let points = sparkline(&metric.trace, rect.shrink2(vec2(0.0, 4.0)));
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(1.4, SIGNAL)));
    ui.label(text);
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

    /// Run the bar row at `width` px; returns the screen-space text shapes.
    fn laid_out(width: f32) -> (Rect, Vec<(String, Rect)>, Vec<Rect>) {
        let ctx = egui::Context::default();
        crate::theme::visuals::install(&ctx);
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(width, 300.0));
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        let mut bar = Rect::NOTHING;
        let out = ctx.run(input, |ctx| {
            egui::TopBottomPanel::bottom("t")
                .exact_height(44.0)
                .frame(Frame::new().inner_margin(Margin::symmetric(MARGIN as i8, 0)))
                .show(ctx, |ui| {
                    bar = ui.max_rect();
                    ui.horizontal_centered(|ui| {
                        tail(ui, None, Some(10.0));
                    });
                });
        });
        let mut texts = Vec::new();
        let mut rects = Vec::new();
        for s in &out.shapes {
            match &s.shape {
                egui::Shape::Text(t) => {
                    texts.push((t.galley.text().to_string(), t.visual_bounding_rect()))
                }
                egui::Shape::Rect(r) => rects.push(r.visual_bounding_rect()),
                _ => {}
            }
        }
        (bar, texts, rects)
    }

    #[test]
    fn rate_selector_stays_on_one_row_inside_the_bar() {
        for width in [700.0, 1400.0] {
            let (bar, texts, rects) = laid_out(width);
            let panel = Rect::from_min_max(pos2(0.0, bar.top()), pos2(width, bar.bottom()));
            assert_eq!(panel.height(), 44.0);
            let labels: Vec<_> = RATE_PRESETS.iter().map(|p| format!("{p:.0}")).collect();
            let rows: Vec<Rect> = labels
                .iter()
                .map(|l| texts.iter().find(|(t, _)| t == l).expect("label drawn").1)
                .collect();
            assert!(rows.windows(2).all(|w| w[0].min.y == w[1].min.y));
            for r in rows.iter().chain(rects.iter().filter(|r| r.is_positive())) {
                assert!(
                    panel.contains_rect(*r),
                    "{r:?} outside {panel:?} at {width}"
                );
            }
        }
    }
}
