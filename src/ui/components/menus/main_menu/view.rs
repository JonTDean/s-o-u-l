//! Top-level main menu *view*.
//!
//! Lives in its own file so `main_menu::mod.rs` can re-export it and callers
//! can keep writing `main_menu::MainMenu`.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2};

use crate::{
    app_state::AppState,
    ui::components::menus::{style, MenuScreen},
};

/// Renders the visible main menu while [`AppState::MainMenu`] is active.
///
/// Zero-sized → trivially `Send + Sync`, so we can store it as a Bevy
/// `Resource` without extra ceremony.
#[derive(Default, Resource)]
pub struct MainMenu;

impl MenuScreen for MainMenu {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        egui::Window::new("S.O.U.L – Swarm Orchestrator for Autonomous Learners")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .frame(style::window_frame())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Order mirrors the enum for muscle memory.
                    if ui.button("New Scenario").clicked()  { next.set(AppState::NewScenario); }
                    if ui.button("Load Scenario").clicked() { next.set(AppState::LoadScenario); }
                    if ui.button("Options").clicked()       { next.set(AppState::Options);      }
                    if ui.button("Quit").clicked()          { std::process::exit(0);            }
                });
            });
    }
}
