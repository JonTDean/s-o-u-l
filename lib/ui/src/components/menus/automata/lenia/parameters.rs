use bevy::prelude::*;
use bevy_egui::egui;
use engine_core::automata::lenia::{LeniaAction, LeniaCommand, LeniaSimulation};
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};

pub(super) fn show(
    ui: &mut egui::Ui,
    entity: Entity,
    sim: &LeniaSimulation,
    commands: &mut EventWriter<LeniaCommand>,
) {
    let mut seed = sim.seed;
    ui.horizontal(|ui| {
        ui.label("Seed:");
        ui.selectable_value(&mut seed, LeniaSeed::Blob, "Blob");
        ui.selectable_value(&mut seed, LeniaSeed::Ring, "Ring");
    });
    if seed != sim.seed {
        commands.write(LeniaCommand {
            entity,
            action: LeniaAction::Seed(seed),
        });
    }
    let mut params = sim.params;
    ui.add(egui::Slider::new(&mut params.mu, 0.0..=1.0).text("Growth center"));
    ui.add(
        egui::Slider::new(&mut params.sigma, 0.001..=1.0)
            .logarithmic(true)
            .text("Growth width"),
    );
    ui.add(egui::Slider::new(&mut params.dt, 0.0..=1.0).text("Time step"));
    if ui.button("Restore growth defaults").clicked() {
        params = LeniaParams::default();
    }
    if params != sim.params {
        commands.write(LeniaCommand {
            entity,
            action: LeniaAction::Parameters(params),
        });
    }
}
