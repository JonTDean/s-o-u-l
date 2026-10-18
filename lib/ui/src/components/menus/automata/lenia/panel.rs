use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use engine_core::automata::lenia::{LeniaAction, LeniaCommand, LeniaSimulation};

pub(super) fn show(
    mut contexts: EguiContexts,
    simulations: Query<(Entity, &LeniaSimulation)>,
    mut commands: EventWriter<LeniaCommand>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if simulations.is_empty() {
        return;
    }
    egui::Window::new("Lenia workbench")
        .default_pos([12.0, 130.0])
        .default_width(280.0)
        .resizable(false)
        .show(ctx, |ui| {
            let mut owners: Vec<_> = simulations.iter().collect();
            owners.sort_by_key(|(entity, _)| entity.to_bits());
            for (entity, sim) in owners {
                ui.push_id(entity, |ui| {
                    ui.label(format!(
                        "Instance {} / requested tick {}",
                        entity.index(),
                        sim.target_tick
                    ));
                    ui.label(if sim.paused { "Paused" } else { "Running" });
                    ui.horizontal(|ui| {
                        let mut send = |action| {
                            commands.write(LeniaCommand { entity, action });
                        };
                        if ui
                            .button(if sim.paused { "Resume" } else { "Pause" })
                            .clicked()
                        {
                            send(LeniaAction::Pause(!sim.paused));
                        }
                        if ui
                            .add_enabled(sim.paused, egui::Button::new("Single step"))
                            .clicked()
                        {
                            send(LeniaAction::Step);
                        }
                        if ui.button("Reset").clicked() {
                            send(LeniaAction::Reset);
                        }
                    });
                    ui.separator();
                });
            }
            ui.small("XY planes evolve independently; zero-density boundaries.");
        });
}
