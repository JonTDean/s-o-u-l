//! Instrument visual language: tokens, Libertinus fonts, borderless egui style, widgets.
pub mod fade;
pub mod fonts;
pub mod tokens;
pub mod visuals;
pub mod widgets;
pub use visuals::{BOTTOM_PAD, fullscreen_bg, panel_bg};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

/// Installs the theme into the primary egui context on its first pass, before any UI draws.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(fade::FadePlugin)
            .add_systems(EguiPrimaryContextPass, install_once.in_set(ThemeSet));
    }
}

/// Ordering label so the lab draws after the theme is installed.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThemeSet;

fn install_once(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut() {
        visuals::install(ctx);
        *done = true;
    }
}
