//! Top-level module for UI menus.
//!
//! * Removes all traces of the deprecated **`UiCameraConfig`** component.
//! * Spawns a dedicated *UI camera* using the modern `Camera2d` component
//!   (all required components are inserted automatically).
//!
//! The UI camera renders **after** the world camera by giving it a larger
//! `Camera::order` value.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use engine_core::state::AppState;

pub mod main_menu;
pub mod file_io;
pub mod world;

/// Trait implemented by every concrete *menu screen* type.
pub trait MenuScreen: Send + Sync + 'static {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>);
}

/// Spawns the single **UI camera** at application start-up.
///
/// * `Camera2d` activates the 2-D render graph.
/// * `Camera { order: 100 }` ensures the UI camera renders **last**,
///   compositing all menus and HUD elements above the simulation.
pub(crate) fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera { order: 100, ..default() },
    ));
}

/// Drives any [`MenuScreen`] implementor while its associated state is active.
pub(crate) fn ui_runner<T: MenuScreen + Resource>(
    mut screen: ResMut<T>,
    mut contexts: EguiContexts,
    mut next: ResMut<NextState<AppState>>,
) {
    let ctx = contexts
        .ctx_mut()
        .expect("Egui primary window context not found");
    screen.ui(ctx, &mut next);
}
