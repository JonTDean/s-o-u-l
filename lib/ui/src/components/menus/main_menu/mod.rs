//! Main-menu *view* & resource.
//!
//! Renders four centred buttons and emits `NextState<AppState>`
//! transitions.  No game logic lives here.

use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, Frame};
use engine_core::prelude::AppState;
use crate::{components::menus::MenuScreen, styles};

pub mod plugin;

#[derive(Default, Resource)]
pub struct MainMenuScreen;     // zero-sized, but keeps type safety

impl MenuScreen for MainMenuScreen {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>) {
        // fullscreen grey background
        egui::CentralPanel::default().frame(styles::fullscreen_bg()).show(ctx, |_| {});

        // 30 % from bottom
        let offset_y = -(ctx.screen_rect().height() * 0.30).min(-8.0);

        egui::Area::new("main_menu_btns".into())
            .anchor(Align2::CENTER_BOTTOM, [0.0, offset_y])
            .show(ctx, |ui| {
                Frame::none().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    ui.vertical_centered(|ui| {
                        if ui.button("New Scenario").clicked()  { next.set(AppState::NewScenario); }
                        if ui.button("Load Scenario").clicked() { next.set(AppState::LoadScenario); }
                        if ui.button("Options").clicked()       { next.set(AppState::Options); }
                        if ui.button("Quit").clicked()          { std::process::exit(0);        }
                    });
                });
            });
    }
}
