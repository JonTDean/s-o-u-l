//! Lists every *.json* in Documents/SOUL/saves and loads on click.
use bevy::prelude::*;
use bevy_egui::egui::{self, Align2};
use engine::{core::world::World2D, systems::state::{resources::doc_dir, AppState}};
use serde::{Deserialize, Serialize};
use simulation_kernel::grid::GridBackend;
use std::{fs, path::PathBuf};

use crate::{panels::{main_menu::controller::scenario::new::ScenarioMeta, MenuScreen}, styles};


/* ---------- on‑disk format (must match saver) -------------------------- */

#[derive(Serialize, Deserialize)]
struct SavedScenario {
    backend:   GridBackend,
    cell_size: f32,
    bg_color:  [f32; 4],
    params:    ScenarioMeta,
}

/* ---------- screen resource ------------------------------------------- */

#[derive(Resource, Default)]
pub struct LoadScenario {
    files:    Vec<PathBuf>,
    selected: Option<PathBuf>,
}

impl LoadScenario {
    fn refresh(&mut self) {
        let dir = doc_dir().join("saves");
        self.files = fs::read_dir(&dir)
            .map(|it| {
                it.filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |e| e == "json"))
                    .collect()
            })
            .unwrap_or_default();
    }
}

/* ---------- UI --------------------------------------------------------- */

impl MenuScreen for LoadScenario {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        self.refresh(); // inexpensive – call every frame

        egui::CentralPanel::default()
            .frame(styles::fullscreen_bg())
            .show(ctx, |ui| {
                egui::Window::new("S.O.U.L. – Load Scenario")
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .resizable(false)
                    .frame(styles::fullscreen_bg())
                    .show(ui.ctx(), |ui| {
                        if self.files.is_empty() {
                            ui.label("No save files found.");
                        } else {
                            for path in &self.files {
                                let label = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("Unnamed");
                                if ui.button(label).clicked() {
                                    self.selected = Some(path.clone());
                                }
                            }
                        }
                        ui.separator();
                        if ui.button("Back").clicked() {
                            next.set(AppState::MainMenu);
                        }
                    });
            });
    }
}

/* ---------- loader system --------------------------------------------- */

pub fn load_selected_save(
    mut commands: Commands,
    mut load:     ResMut<LoadScenario>,
    mut next:     ResMut<NextState<AppState>>,
) {
    let Some(path) = load.selected.take() else { return };

    let Ok(bytes)    = fs::read(&path) else {
        eprintln!("Read error: {}", path.display());
        return;
    };
    let Ok(snapshot) = serde_json::from_slice::<SavedScenario>(&bytes) else {
        eprintln!("Parse error");
        return;
    };

    let bg = Color::srgba(
        snapshot.bg_color[0],
        snapshot.bg_color[1],
        snapshot.bg_color[2],
        snapshot.bg_color[3],
    );

    commands.insert_resource(World2D {
        backend:   snapshot.backend,
        cell_size: snapshot.cell_size,
        bg_color:  bg,
    });
    commands.insert_resource(snapshot.params);

    next.set(AppState::InGame);
}