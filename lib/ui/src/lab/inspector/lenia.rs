//! Lenia growth sliders, seed choice and the opt-in GPU mass toggle.
use crate::{
    lab::Effects,
    theme::widgets::{choice, margin_section, note, serif_italic, slider},
};
use bevy_egui::egui::{self, Ui};
use engine_core::automata::{
    channel::{
        ChannelAction, ChannelCommand, ChannelId,
        metrics::{ChannelMetric, MetricAvailability},
    },
    lenia::{LeniaAction, LeniaSimulation},
};
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};

/// Growth, seed and sampling sections for one Lenia channel.
pub fn show(
    ui: &mut Ui,
    id: ChannelId,
    sim: &LeniaSimulation,
    metric: Option<&ChannelMetric>,
    fx: &mut Effects,
) {
    let send = |fx: &mut Effects, action| fx.commands.push(ChannelCommand { id, action });
    margin_section(ui, "growth", false, |ui| {
        let mut params = sim.params;
        slider(ui, "μ", &mut params.mu, 0.0..=1.0, false);
        slider(ui, "σ", &mut params.sigma, 0.001..=1.0, true);
        slider(ui, "Δt", &mut params.dt, 0.0..=1.0, false);
        if ui.link(serif_italic("restore defaults")).clicked() {
            params = LeniaParams::default();
        }
        if params != sim.params {
            send(fx, ChannelAction::Lenia(LeniaAction::Parameters(params)));
        }
    });
    margin_section(ui, "seed", false, |ui| {
        let mut seed = sim.seed;
        let options = [(LeniaSeed::Blob, "blob"), (LeniaSeed::Ring, "ring")];
        if choice(ui, &mut seed, &options) {
            send(fx, ChannelAction::Lenia(LeniaAction::Seed(seed)));
        }
    });
    margin_section(ui, "sample", false, |ui| {
        let availability = metric.map(|m| &m.availability);
        let mut on = !matches!(availability, None | Some(MetricAvailability::OptIn));
        let label = serif_italic("mass (GPU readback)");
        if ui.add(egui::Checkbox::new(&mut on, label)).changed() {
            send(fx, ChannelAction::SampleMetric(on));
        }
        if let Some(MetricAvailability::Unavailable(reason)) = availability {
            note(ui, reason, true);
        }
    });
}
