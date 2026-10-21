use super::*;
use crate::theme::widgets::{note, pill};
use bevy_egui::egui::{self, Stroke};
use engine_core::automata::channel::{ChannelAction, ChannelCommand, metrics::MetricAvailability};
pub(super) fn trace(ui: &mut Ui, id: ChannelId, data: &LabData, fx: &mut Effects) {
    let Some(metric) = data.metrics.get(id) else {
        return;
    };
    if matches!(metric.availability, MetricAvailability::OptIn)
        && pill(ui, "Sample mass", false, true).clicked()
    {
        fx.commands.push(ChannelCommand {
            id,
            action: ChannelAction::SampleMetric(true),
        });
    }
    let samples: Vec<_> = metric.trace.samples().collect();
    let (Some(first), Some(last), Some((lo, hi))) =
        (samples.first(), samples.last(), metric.trace.range())
    else {
        return;
    };
    note(
        ui,
        &format!(
            "{} · {}\nepoch {} · ticks {}–{}\n{lo:.3}–{hi:.3}",
            metric.id.label, metric.id.unit, last.epoch, first.tick, last.tick
        ),
        false,
    );
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 64.0), egui::Sense::hover());
    let points = samples
        .iter()
        .map(|s| {
            egui::pos2(
                rect.left()
                    + rect.width() * (s.tick - first.tick) as f32
                        / (last.tick - first.tick).max(1) as f32,
                rect.bottom()
                    - rect.height()
                        * if hi > lo {
                            ((s.value - lo) / (hi - lo)) as f32
                        } else {
                            0.5
                        },
            )
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(1.2, SIGNAL)));
}
mod heatmap;
pub(super) use heatmap::difference;
