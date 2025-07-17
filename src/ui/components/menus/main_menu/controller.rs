//! Controller layer for the main-menu option screens.
//!
//! * Implements [`crate::ui::components::menus::MenuScreen`].  
//! * Owns a private **model** struct for each screen and mutates it
//!   in response to user input.
//!
//! The controllers themselves are Bevy `Resource`s so they can store
//! local state (e.g. text-field contents) across frames.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2};

use crate::{
    app_state::AppState,
    ui::components::menus::{style, MenuScreen},
};

use super::model::{LoadScenarioData, ScenarioDraft, UiSettings};

/// ───────────────────────── “New Scenario” ───────────────────────────────────
#[derive(Resource, Default)]
pub struct NewScenario {
    model: ScenarioDraft,
}

impl MenuScreen for NewScenario {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        egui::Window::new("S.O.U.L. – New Scenario")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .frame(style::window_frame())
            .show(ctx, |ui| {
                ui.label("Board size (cells)");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.model.width).range(8..=512));
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut self.model.height).range(8..=512));
                });

                ui.separator();

                if ui.button("Start simulation").clicked() {
                    // World instantiation is deferred to a normal Bevy schedule
                    // so we don’t block the UI thread.
                    next.set(AppState::InGame);
                }

                if ui.button("Back").clicked() {
                    next.set(AppState::MainMenu);
                }
            });
    }
}

/// ───────────────────────── “Load Scenario” ──────────────────────────────────
#[derive(Resource, Default)]
pub struct LoadScenario {
    _model: LoadScenarioData, // kept for future use
}

impl MenuScreen for LoadScenario {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        egui::Window::new("S.O.U.L. – Load Scenario")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .frame(style::window_frame())
            .show(ctx, |ui| {
                ui.label("TODO: file picker + deserialize world");

                if ui.button("Back").clicked() {
                    next.set(AppState::MainMenu);
                }
            });
    }
}

/// ─────────────────────────── “Options” ──────────────────────────────────────
#[derive(Resource, Default)]
pub struct OptionsScreen {
    model: UiSettings,
}

impl MenuScreen for OptionsScreen {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        egui::Window::new("S.O.U.L. – Options")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .frame(style::window_frame())
            .show(ctx, |ui| {
                ui.label("UI scale");
                ui.add(
                    egui::DragValue::new(&mut self.model.font_size).range(8.0..=32.0),
                );
                ui.separator();

                if ui.button("Back").clicked() {
                    next.set(AppState::MainMenu);
                }
            });
    }
}
