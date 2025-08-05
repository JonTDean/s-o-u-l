//! Main-menu scene – minimal, self-contained and schedule-aware.
//!
//! * Appears whenever [`AppState::MainMenu`] is active.
//! * Runs in [`MainSet::Render`] so the UI is built _after_ game logic.
//! * Relies only on **bevy_egui**; there are **no** dependencies on the
//!   heavier UI crate, so this module is safe in headless test builds.

use bevy::prelude::*;
use engine_core::{
    prelude::AppState,
};

use crate::scenes::hide_world_camera;

/// Public plug-in – add once, forget forever.
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), hide_world_camera)
           .add_systems(OnExit(AppState::MainMenu),  hide_world_camera);
    }
}
