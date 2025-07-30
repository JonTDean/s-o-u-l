//! Main‑menu *view*.
//!
//! • Renders the four primary buttons while the app is in [`AppState::MainMenu`].  
//! • The block is horizontally centred and kept 30 % above the bottom edge.  
//! • The buttons are ordered: _New Scenario_, _Load Scenario_, _Options_, _Quit_.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, Frame};
use engine::systems::state::AppState;

use crate::{panels::MenuScreen, styles};

/// Zero‑sized resource that drives the UI.
#[derive(Default, Resource)]
pub struct MainMenu;

impl MenuScreen for MainMenu {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        // ── 1. paint background ─────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(styles::fullscreen_bg())
            .show(ctx, |_| {});

        // ── 2. dynamic vertical offset: 30 % of window height ───────────────
        let offset_y = -(ctx.screen_rect().height() * 0.30).max(8.0);

        // ── 3. button column inside a frameless “div” ───────────────────────
        egui::Area::new("main_menu_buttons".into())
            .anchor(Align2::CENTER_BOTTOM, [0.0, offset_y])
            .show(ctx, |ui| {
                Frame::new()                         // ← no border / drop‑shadow
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;

                        ui.vertical_centered(|ui| {
                            // order reversed per your request ⮑
                            if ui.button("New Scenario").clicked()  { next.set(AppState::NewScenario); }
                            if ui.button("Load Scenario").clicked() { next.set(AppState::LoadScenario); }
                            if ui.button("Options").clicked()       { next.set(AppState::Options); }
                            if ui.button("Quit").clicked()          { std::process::exit(0); }
                        });
                    });
            });
    }
}
