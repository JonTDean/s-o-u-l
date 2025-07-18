//! UI for the in-game side panel, providing simulation controls while in [`AppState::InGame`].
#![allow(unused_imports)]

use bevy::prelude::*;
use bevy_egui::{egui::{self, Align2}, EguiPrimaryContextPass};

use crate::{
    state::AppState,
    intelligence_engine::core::World2D,
    ui::components::menus::{MenuScreen, ui_runner},
    ui::styles,
};

/// Resource representing the in-game side panel (no internal state fields needed for now).
#[derive(Resource, Default)]
pub struct SidePanel;

/// Bevy Plugin to manage the in-game side panel UI.
pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app
            // When entering InGame, add the side panel resource
            .add_systems(OnEnter(AppState::InGame), |mut commands: Commands| {
                commands.insert_resource(SidePanel::default());
            })
            // While in InGame, run the side panel UI each frame (using Egui context)
            .add_systems(
                EguiPrimaryContextPass,
                ui_runner::<SidePanel>.run_if(in_state(AppState::InGame)),
            )
            // On exiting InGame, remove the side panel and World resources (cleanup)
            .add_systems(OnExit(AppState::InGame), |mut commands: Commands| {
                commands.remove_resource::<SidePanel>();
                commands.remove_resource::<World2D>();
            });
    }
}

impl MenuScreen for SidePanel {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        // Side panel docked to the left side of the window
        egui::SidePanel::left("in_game_side_panel")
            .min_width(200.0)
            .resizable(false)
            .frame(styles::panel_bg())
            .show(ctx, |ui| {
                ui.heading("Simulation Controls");
                ui.separator();
                if ui.button("Back to Main Menu").clicked() {
                    // Transition back to main menu state
                    next.set(AppState::MainMenu);
                }
                // Future control buttons (pause, resume, etc.) can be added here.
            });
    }
}
