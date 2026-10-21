//! Presenter mode: full-window stage, HUD readouts, plain title, fading transport.
use super::paint;
use crate::{
    lab::{Effects, LabData, inspector::sections::format_metric, state::LabState},
    theme::{
        tokens::*,
        widgets::{dim, mono, pill, plain, tick_digits},
    },
};
use bevy_egui::egui::{self, Align2, Frame};
use engine_common::controls::camera::manager::stage::StageRect;
use engine_core::automata::channel::{ChannelAction, ChannelCommand, ChannelRow, Status};

/// Draw presenter mode and return the full-window stage rectangle.
pub fn show(
    ctx: &egui::Context,
    lab: &mut LabState,
    data: &LabData,
    fx: &mut Effects,
    typing: bool,
) -> StageRect {
    let rect = egui::CentralPanel::default()
        .frame(Frame::NONE)
        .show(ctx, |ui| paint(ui, lab, data, typing, false, fx))
        .inner;
    let Some(row) = data.channels.selected_row() else {
        return rect;
    };
    egui::Area::new("lab.hud".into())
        .anchor(Align2::RIGHT_TOP, [-20.0, 16.0])
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                ui.label(mono(tick_digits(row.tick)).size(size::HUD));
                ui.label(dim("tick"));
                let metric = data.metrics.get(row.id);
                if let Some((m, sample)) = metric.and_then(|m| m.latest.map(|s| (m, s))) {
                    ui.label(mono(format_metric(sample.value, &m.id)).size(26.0));
                    ui.label(dim(m.id.label));
                }
            });
        });
    if let Some(specimen) = data.specimens.get(&row.kind) {
        egui::Area::new("lab.title".into())
            .anchor(Align2::LEFT_BOTTOM, [22.0, -18.0])
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                let n = data.specimens.figure(&row.kind).unwrap_or(0);
                let heading = format!("Specimen {n} · {}", specimen.group.label());
                ui.label(dim(heading).size(size::BODY + 1.0));
                let session = data.bench.sessions.iter().find(|s| s.channel == row.id);
                let title = crate::lab::chrome::tabs::title(row, &data.specimens, session);
                ui.label(plain(title).size(24.0).color(INK_HI));
                ui.label(dim(&specimen.caption));
            });
    }
    let alpha = lab.transport_alpha();
    if alpha > 0.0 {
        egui::Area::new("lab.mini".into())
            .anchor(Align2::RIGHT_BOTTOM, [-20.0, -18.0])
            .show(ctx, |ui| {
                ui.multiply_opacity(alpha * 0.8);
                ui.horizontal(|ui| mini_transport(ui, row, fx));
            });
    }
    rect
}

fn mini_transport(ui: &mut egui::Ui, row: &ChannelRow, fx: &mut Effects) {
    let running = row.status == Status::Running;
    let mut send = |action| fx.commands.push(ChannelCommand { id: row.id, action });
    let label = if running { "‖ Pause" } else { "▶ Run" };
    if pill(ui, label, true, row.status != Status::Halted).clicked() {
        send(ChannelAction::Pause(running));
    }
    if pill(ui, "Step", false, row.status == Status::Paused).clicked() {
        send(ChannelAction::Step);
    }
    if pill(ui, "Reset", false, true).clicked() {
        send(ChannelAction::Reset);
    }
}
