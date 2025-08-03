use bevy::prelude::*;
use bevy::ecs::schedule::IntoScheduleConfigs; // <- brings `.run_if` / `.in_set` into scope
use bevy_egui::EguiPrimaryContextPass;
use engine_core::prelude::{AppState, in_state};

use crate::components::debug::{
    debug_menu::debug_menu,
    debug_stats_metrics::debug_stats_metrics,
};

pub struct DebugComponentsPlugin;

impl Plugin for DebugComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            // `run_if` goes on the *system configs* tuple, not on the returned App
            (
                debug_menu,
                debug_stats_metrics,
            )
            .run_if(in_state(AppState::InGame)),
        );
    }
}
