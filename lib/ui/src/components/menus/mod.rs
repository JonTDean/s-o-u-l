use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::prelude::AppState;

pub mod main_menu;
pub mod options_menu;
pub mod plugin;

/// Trait implemented by every concrete *menu screen* type.
pub trait MenuScreen: Send + Sync + 'static {
    fn ui(&mut self, ctx: &egui::Context, next: &mut NextState<AppState>);
}

/// Drives any [`MenuScreen`] implementor while its associated state is active.
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
