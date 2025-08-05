use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::prelude::AppState;

pub mod automata;
pub mod meta;

pub mod plugin;

/// Trait implemented by every concrete *menu screen* type.
///
/// Menus live **exclusively** in the UI layer and communicate with the engine
/// via `NextState<AppState>` only – this guarantees that the ECS world remains
/// fully deterministic.
pub trait MenuScreen: Send + Sync + 'static {
    /// Render the UI and optionally queue a state transition.
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>);
}

/// Generic driver that keeps a menu’s `Resource` alive while its state is
/// active and feeds it the current `egui::Context`.
pub(crate) fn menu_runner<T: MenuScreen + Resource>(
    mut screen: ResMut<T>,
    mut contexts: EguiContexts,
    mut next: ResMut<NextState<AppState>>,
) {
    let ctx = contexts
        .ctx_mut()
        .expect("Egui primary window context not found");
    screen.ui(ctx, &mut next);
}
