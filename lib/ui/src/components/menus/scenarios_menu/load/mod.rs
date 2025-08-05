//! *Load Scenario* menu – lists discovered `*.phsave` files and lets the user
//! resume one.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, CentralPanel, Frame, ScrollArea};
use engine_core::prelude::AppState;

pub mod plugin;

use crate::{components::menus::MenuScreen, styles};
use std::path::PathBuf;

/// UI-only state buffer.
#[derive(Resource)]
pub struct LoadScenarioMenuScreen {
    files:     Vec<PathBuf>,
    selected:  Option<usize>,
}

impl LoadScenarioMenuScreen {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self { files, selected: None }
    }
}

impl MenuScreen for LoadScenarioMenuScreen {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        CentralPanel::default().frame(styles::fullscreen_bg()).show(ctx, |_| {});

        egui::Area::new("load_scenario_win".into())
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                Frame::default().show(ui, |ui| {
                    ui.set_width(450.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("Load Scenario");
                        ui.separator();

                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            if self.files.is_empty() {
                                ui.label("No saved scenarios found.");
                            } else {
                                for (idx, path) in self.files.iter().enumerate() {
                                    let label = path.file_stem()
                                                    .and_then(|s| s.to_str())
                                                    .unwrap_or("unknown");
                                    let selected = self.selected == Some(idx);
                                    if ui.selectable_label(selected, label).clicked() {
                                        self.selected = Some(idx);
                                    }
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(self.selected.is_some(), egui::Button::new("Load"))
                                .clicked()
                            {
                                // Real loading will occur in the engine layer.  For now we just hop in.
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
