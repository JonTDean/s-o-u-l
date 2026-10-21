//! Lenia growth sliders, seed choice and the opt-in GPU mass toggle.
use crate::{
    lab::Effects,
    theme::widgets::{dim, margin_section, note, segmented, slider},
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
    margin_section(ui, "model", false, |ui| {
        use simulation_kernel::core::{
            boundary::Boundary,
            lenia::continuous::{Kernel, Shape},
        };
        let mut continuous = sim.kernel.is_some();
        segmented(
            ui,
            &mut continuous,
            &[(false, "quantized XY"), (true, "continuous density")],
        );
        let mut kernel = sim.kernel.unwrap_or_default();
        if continuous {
            ui.horizontal(|ui| {
                ui.label("radius");
                if kernel.shape == Shape::Orbium {
                    ui.label("13 · periodic XY");
                } else {
                    ui.add(egui::DragValue::new(&mut kernel.radius).range(1..=4));
                }
            });
            segmented(
                ui,
                &mut kernel.shape,
                &[
                    (Shape::Gaussian, "Gaussian XYZ"),
                    (Shape::Shell, "shell XYZ"),
                    (Shape::Orbium, "Orbium XY"),
                ],
            );
            if kernel.shape == Shape::Orbium {
                kernel = simulation_kernel::core::lenia::continuous::orbium::KERNEL;
            } else {
                kernel.radius = kernel.radius.min(4);
            }
            segmented(
                ui,
                &mut kernel.boundary,
                &[(Boundary::Dead, "dead"), (Boundary::Periodic, "periodic")],
            );
        }
        let selected: Option<Kernel> = continuous.then_some(kernel);
        if selected != sim.kernel {
            send(fx, ChannelAction::Lenia(LeniaAction::Kernel(selected)));
        }
        note(
            ui,
            "Model changes reset. Gaussian/shell couple XYZ; Orbium uses polynomial XY. Select the Orbium seed to restore published parameters.",
            false,
        );
    });
    margin_section(ui, "growth", false, |ui| {
        let mut params = sim.params;
        slider(ui, "μ", &mut params.mu, 0.0..=1.0, false);
        slider(ui, "σ", &mut params.sigma, 0.001..=1.0, true);
        slider(ui, "Δt", &mut params.dt, 0.0..=1.0, false);
        if ui.link(dim("restore defaults")).clicked() {
            params = LeniaParams::default();
        }
        if params != sim.params {
            send(fx, ChannelAction::Lenia(LeniaAction::Parameters(params)));
        }
    });
    margin_section(ui, "seed", false, |ui| {
        let mut seed = sim.seed;
        let options = [
            (LeniaSeed::Blob, "blob"),
            (LeniaSeed::Ring, "ring"),
            (LeniaSeed::Orbium, "published Orbium"),
        ];
        if segmented(ui, &mut seed, &options) {
            send(fx, ChannelAction::Lenia(LeniaAction::Seed(seed)));
        }
    });
    margin_section(ui, "sample", false, |ui| {
        let availability = metric.map(|m| &m.availability);
        let mut on = !matches!(availability, None | Some(MetricAvailability::OptIn));
        let label = dim("mass (GPU readback)");
        if ui.add(egui::Checkbox::new(&mut on, label)).changed() {
            send(fx, ChannelAction::SampleMetric(on));
        }
        if let Some(MetricAvailability::Unavailable(reason)) = availability {
            note(ui, reason, true);
        }
    });
}
