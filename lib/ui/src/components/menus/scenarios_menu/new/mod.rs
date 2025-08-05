//! *New Scenario* menu – collects a scenario name from the player and
//! transitions straight into `AppState::InGame`.  Heavy lifting (map
//! generation, asset streaming) is done by the engine layer.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, CentralPanel, Frame};
use engine_core::prelude::AppState;

use crate::{components::menus::MenuScreen, styles};

pub mod plugin;

/// UI-only state buffer.
#[derive(Resource, Default)]
pub struct NewScenarioMenuScreen {
    name: String,
}

impl MenuScreen for NewScenarioMenuScreen {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        CentralPanel::default().frame(styles::fullscreen_bg()).show(ctx, |_| {});

        egui::Area::new("new_scenario_win".into())
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                Frame::default().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Create New Scenario");
                        ui.separator();
                        ui.label("Scenario name:");
                        ui.text_edit_singleline(&mut self.name);
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(!self.name.trim().is_empty(), egui::Button::new("Start"))
                                .clicked()
                            {
                                // A real implementation would stash the draft into a resource that
                                // the simulation layer consumes.  For now, we jump straight in.
                                next.set(AppState::InGame);
                            }
                            if ui.button("Back").clicked() {
                                next.set(AppState::MainMenu);
                            }
                        });
                    });
                });
            });
    }
}
