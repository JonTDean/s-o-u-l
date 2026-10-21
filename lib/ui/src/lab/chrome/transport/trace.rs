//! Metric trace pieces: sparkline geometry, rate matching and the trace widget.
use crate::{
    lab::inspector::sections::format_metric,
    theme::{
        fonts::{MONO_MEDIUM, family},
        tokens::*,
        widgets::{caps, mono},
    },
};
use bevy_egui::egui::{self, Pos2, Rect, Sense, Stroke, pos2, vec2};
use engine_core::automata::channel::{
    RATE_PRESETS,
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

/// Metric label, sparkline filling the space, and the latest value.
pub(super) fn trace(ui: &mut egui::Ui, metric: Option<&ChannelMetric>) {
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
}
