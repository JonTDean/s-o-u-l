//! “Spawn pattern” window – issues `AutomataCommand::SeedPattern` events.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::events::AutomataCommand;
use crate::ui::panels::main_menu::controller::scenario::new::ScenarioMeta;

pub fn spawn_panel(
    selected: Res<ScenarioMeta>,
    mut egui_ctx: EguiContexts,
    mut writer:   EventWriter<AutomataCommand>,
) {
    let ctx = egui_ctx.ctx_mut().unwrap();
    egui::Window::new("Spawner")
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Spawn one pattern:");
            ui.separator();
            for id in selected
                .0
                .selected_classical
                .iter()
                .chain(selected.0.selected_dynamical.iter())
            {
                if ui.button(format!("➕ {id}")).clicked() {
                    writer.write(AutomataCommand::SeedPattern { id: id.clone() });
                }
            }
        });
}
