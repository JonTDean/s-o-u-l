//! “Spawn pattern” window – issues `AutomataCommand::SeedPattern` events.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::{
    events::{AutomataCommand, GenerateDebugFloor},
    systems::registry::RuleRegistry,
};

/// Renders the small “Spawner” popup and turns button clicks into events.
pub fn spawner(
    rules:      Res<RuleRegistry>,                // ← existing global registry
    mut egui:   EguiContexts,
    mut cmds:   EventWriter<AutomataCommand>,
    mut floor:  EventWriter<GenerateDebugFloor>,
) {
    let ctx = egui.ctx_mut().unwrap();

    egui::Window::new("Spawner")
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Spawn one pattern:");
            ui.separator();

            // classical + dynamical rules are all in the same registry
            for id in rules.ids() {
                if ui.button(format!("➕ {id}")).clicked() {
                    cmds.write(AutomataCommand::SeedPattern { id: id.clone() });
                }
            }

            ui.separator();
            if ui.button("◻ Generate Debug Floor").clicked() {
                floor.write(GenerateDebugFloor);
            }
        });
}
