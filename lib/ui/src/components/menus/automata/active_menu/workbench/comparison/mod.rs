//! Independent pair selection, synchronized stepping and isolated export jobs.
use bevy::prelude::*;
mod exports;
mod panel;
pub struct ComparisonPlugin;
impl Plugin for ComparisonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Comparison>().add_systems(
            bevy_egui::EguiPrimaryContextPass,
            panel::show.run_if(in_state(engine_core::prelude::AppState::InGame)),
        );
    }
}
#[derive(Resource)]
pub struct Comparison {
    pub left: usize,
    pub right: usize,
    pub folder: String,
    pub status: String,
    pub job: Option<std::thread::JoinHandle<Result<(), String>>>,
}
impl Default for Comparison {
    fn default() -> Self {
        Self {
            left: 0,
            right: 1,
            folder: "soul-export".into(),
            status: String::new(),
            job: None,
        }
    }
}
