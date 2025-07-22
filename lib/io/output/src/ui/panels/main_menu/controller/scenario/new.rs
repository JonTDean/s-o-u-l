//! “New Scenario” configuration screen.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2};
use serde::{Deserialize, Serialize};
use engine_core::{
    state::AppState,
    core::World2D,
    engine::grid::{
        DenseGrid, 
        GridBackend, 
        SparseGrid
    }
};

use crate::ui::{
    styles, 
    panels::{
        MenuScreen, 
        main_menu::model::{
            GridType, 
            Rgba, 
            ScenarioDraft
        }
    }, 
};


/// Screen resource.
#[derive(Resource, Default)] pub struct NewScenario { pub model: ScenarioDraft }

/// Long‑lived copy used by autosave / manual save.
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct ScenarioMeta(pub ScenarioDraft);

impl MenuScreen for NewScenario {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        egui::CentralPanel::default().frame(styles::fullscreen_bg()).show(ctx, |ui| {
            egui::Window::new("S.O.U.L. – New Scenario")
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .frame(styles::fullscreen_bg())
                .show(ui.ctx(), |ui| {

                    // ── Scenario name ─────────────────────────────────────
                    ui.label("Scenario name (file name)");
                    ui.text_edit_singleline(&mut self.model.name);

                    ui.separator();

                    // ── Board size ────────────────────────────────────────
                    ui.label("Board size (cells)");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.model.width ).range(8..=512));
                        ui.label("×");
                        ui.add(egui::DragValue::new(&mut self.model.height).range(8..=512));
                    });

                    ui.separator();

                    // ── Backend type ───────────────────────────────────────
                    ui.label("Grid backend");
                    egui::ComboBox::new("grid_type_combo", "")
                        .selected_text(match self.model.grid_type {
                            GridType::Dense => "Dense", GridType::Sparse => "Sparse"
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.model.grid_type, GridType::Dense,  "Dense");
                            ui.selectable_value(&mut self.model.grid_type, GridType::Sparse, "Sparse");
                        });

                    ui.separator();

                    // ── Cell‑size selector ────────────────────────────────
                    ui.label("Cell size (pixels)");
                    ui.add(egui::DragValue::new(&mut self.model.cell_size)
                        .speed(1.0).range(1.0..=64.0));

                    ui.separator();

                    // ── Background colour ────────────────────────────────
                    ui.label("Background colour");
                    let mut c = egui::Color32::from_rgba_unmultiplied(
                        self.model.bg_color.r, self.model.bg_color.g,
                        self.model.bg_color.b, self.model.bg_color.a);
                    ui.color_edit_button_srgba(&mut c);
                    self.model.bg_color = Rgba { r: c.r(), g: c.g(), b: c.b(), a: c.a() };

                    ui.separator();

                    // ── Automata selection ───────────────────────────────────────
                    ui.label("Automata selection:");
                    
                    // Group 1: Classical Automata
                    ui.collapsing("Classical Automata", |ui| {
                        // Define available classical automata options (ID and label)
                        let classical_options = [
                            ("wolfram:rule30", "Wolfram Rule 30 (1D Elementary)"),
                            ("wolfram:rule110", "Wolfram Rule 110 (1D Elementary)"),
                            // ... you can list other classical automata here ...
                        ];
                        for (id, label) in classical_options {
                            // Determine if this automaton is currently selected
                            let mut selected = self.model.selected_classical.contains(&id.to_string());
                            if ui.checkbox(&mut selected, label).changed() {
                                if selected {
                                    // if checked, add to selection list
                                    if !self.model.selected_classical.iter().any(|x| x == id) {
                                        self.model.selected_classical.push(id.to_string());
                                    }
                                } else {
                                    // if unchecked, remove from selection list
                                    self.model.selected_classical.retain(|x| x != id);
                                }
                            }
                        }
                    });

                    // Group 2: Dynamical Automata
                    ui.collapsing("Dynamical Automata", |ui| {
                        let dynamical_options = [
                            ("life:conway", "Conway's Game of Life (2D)"),
                            ("life:dean", "Dean's Life (2D multi-state)"),
                            ("lenia", "Lenia (3D continuous)"),
                            // ... other dynamical automata ...
                        ];
                        for (id, label) in dynamical_options {
                            let mut selected = self.model.selected_dynamical.contains(&id.to_string());
                            if ui.checkbox(&mut selected, label).changed() {
                                if selected {
                                    if !self.model.selected_dynamical.iter().any(|x| x == id) {
                                        self.model.selected_dynamical.push(id.to_string());
                                    }
                                } else {
                                    self.model.selected_dynamical.retain(|x| x != id);
                                }
                            }
                        }
                    });

                    ui.separator();

                    // ── Action buttons ───────────────────────────────────────
                    if ui.button("Start simulation").clicked() {
                        next.set(AppState::InGame);
                    }
                    if ui.button("Back").clicked() {
                        next.set(AppState::MainMenu);
                    }

                });
        });
    }
}

/// Builds [`World2D`] and copies [`ScenarioMeta`] into the ECS.
pub fn init_new_world(
    mut commands: Commands,
    draft: Res<ScenarioMeta>,
) {
    let m = &draft.0;

    // 1. backend
    let backend = match m.grid_type {
        GridType::Dense  => GridBackend::Dense(DenseGrid::blank(UVec2::new(m.width, m.height))),
        GridType::Sparse => GridBackend::Sparse(SparseGrid::default()),
    };

    // 2. background colour
    let bg = Color::srgba_u8(m.bg_color.r, m.bg_color.g, m.bg_color.b, m.bg_color.a);

    // 3. world resource
    commands.insert_resource(World2D { backend, cell_size: m.cell_size, bg_color: bg });
}
