//! Pause-menu overlay (Esc ⇄ Resume / Main Menu).
use bevy::prelude::*;
use bevy_egui::egui;

use engine_core::state::AppState;
use crate::ui::{panels::MenuScreen, styles};

#[derive(Resource, Default)]
pub struct PauseMenu;

impl MenuScreen for PauseMenu {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        egui::CentralPanel::default()
            .frame(styles::fullscreen_bg())
            .show(ctx, |ui| {
                egui::Window::new("Paused")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .resizable(false)
                    .frame(styles::fullscreen_bg())
                    .show(ui.ctx(), |ui| {
                        ui.vertical_centered(|ui| {
                            if ui.button("Resume").clicked() {
                                next.set(AppState::InGame);
                            }
                            if ui.button("Main Menu").clicked() {
                                next.set(AppState::MainMenu);
                            }
                        });
                    });
            });
    }
}

/// Toggles between *InGame* and *Paused* on Esc.
pub fn esc_toggle_pause(
    keys:    Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    state:   Res<State<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next.set(AppState::Paused),
            AppState::Paused => next.set(AppState::InGame),
            _ => {}
        }
    }
}

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        // always listen for Esc
        app.add_systems(Update, esc_toggle_pause);

        // show the pause UI only while paused
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            crate::ui::panels::ui_runner::<PauseMenu>.run_if(in_state(AppState::Paused)),
        )
        .add_systems(
            OnEnter(AppState::Paused),
            |mut cmd: Commands| cmd.insert_resource(PauseMenu),
        )
        .add_systems(
            OnExit(AppState::Paused),
            |mut cmd: Commands| cmd.remove_resource::<PauseMenu>(),
        );
    }
}
