//! Root module for all UI menus (main menu, in-game HUD, pause, …).

use bevy::prelude::*;
use bevy_egui::{
    egui,                // ← makes every downstream `egui::…` come from *exactly*
                         //    the same crate version that `bevy_egui` uses
    EguiContexts,
    EguiPlugin,
    EguiPrimaryContextPass,
};

use crate::app_state::AppState;

pub mod style;
pub mod main_menu;

// ───────────────────────── Trait every screen must implement ────────────────
pub trait MenuScreen: Resource + Default + Send + Sync + 'static {
    /// Called once when we *enter* the corresponding [`AppState`].
    fn setup(&mut self, _commands: &mut Commands) {}
    /// Drawn every frame while the state is active.
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>);
}

// ───────────────────────── Plugin boilerplate ───────────────────────────────
pub struct MenuPlugin;

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_state::<AppState>()
            .add_systems(Startup, setup_ui_camera)
            // ─────── Views & models registered here ──────────────────────────
            .add_menu::<main_menu::MainMenu>(AppState::MainMenu)
            .add_menu::<main_menu::NewScenario>(AppState::NewScenario)
            .add_menu::<main_menu::LoadScenario>(AppState::LoadScenario)
            .add_menu::<main_menu::OptionsScreen>(AppState::Options);
    }
}

// ───────────────────── Helper extension for ergonomics ──────────────────────
trait AddMenuExt {
    fn add_menu<S: MenuScreen>(&mut self, state: AppState) -> &mut Self;
}

impl AddMenuExt for App {
    fn add_menu<S: MenuScreen>(&mut self, state: AppState) -> &mut Self {
        self
            // Persist the screen as a resource
            .init_resource::<S>()
            // One-shot setup
            .add_systems(
                OnEnter(state),
                |mut screen: ResMut<S>, mut c: Commands| screen.setup(&mut c),
            )
            // Continuous drawing while the state is active
            .add_systems(
                EguiPrimaryContextPass,
                (
                    |mut screen: ResMut<S>,
                     mut egui_ctx: EguiContexts,
                     mut next: ResMut<NextState<AppState>>| {
                        if let Ok(ctx) = egui_ctx.ctx_mut() {
                            screen.ui(ctx, &mut next);
                        }
                    }
                )
                    .run_if(in_state(state)),
            )
    }
}
