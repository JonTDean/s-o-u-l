//! Top-level module for UI menus. Defines the `MenuScreen` trait and common UI systems.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::state::AppState;

pub mod main_menu;
// (Note: The old `style` module is removed; styles are now in `ui::styles`)

/// Trait for all menu/UI screens to implement their UI drawing logic.
pub trait MenuScreen: Send + Sync + 'static {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>);
}

/// Spawns a 2D camera for rendering UI. Should be run at app startup.
pub(crate) fn setup_ui_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

/// Runs a `MenuScreen` UI system each frame while its corresponding state is active.
pub(crate) fn ui_runner<T: MenuScreen + Resource>(
    mut screen: ResMut<T>,
    mut contexts: EguiContexts,
    mut next: ResMut<NextState<AppState>>,
) {
    let ctx = contexts.ctx_mut().expect("Egui primary window context not found");
    screen.ui(ctx, &mut next);
}
