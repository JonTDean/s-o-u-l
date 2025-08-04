//! “Spawn pattern” window – issues `AutomataCommand::SeedPattern` events.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::events::{AutomataCommand, GenerateDebugFloor};

use crate::panels::main_menu::controller::scenario::new::ScenarioMeta;

pub fn spawn_panel(
    selected:    Res<ScenarioMeta>,
    mut egui_ctx: EguiContexts,
    mut cmd_tx:   EventWriter<AutomataCommand>,     // patterns
    mut floor_tx: EventWriter<GenerateDebugFloor>,
) {
    let ctx = egui_ctx.ctx_mut().unwrap();

    egui::Window::new("Spawner")
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Spawn one pattern:");
            ui.separator();

            for id in &selected.0.selected_classical {
                if ui.button(format!("➕ {id}")).clicked() {
                    cmd_tx.write(AutomataCommand::SeedPattern { id: id.clone() });
                }
            }

            if let Some(ref dyn_id) = selected.0.selected_dynamical {
                if ui.button(format!("➕ {dyn_id}")).clicked() {
                    cmd_tx.write(AutomataCommand::SeedPattern { id: dyn_id.clone() });
                }
            }

            if ui.button("◻ Generate Debug Floor").clicked() {
                floor_tx.write(GenerateDebugFloor);
            }
        });
}
