//! Project controls and asynchronous completed-state capture.
use bevy::prelude::*;
use engine_core::automata::lenia::LeniaSimulation;
use models::workbench::project::Project;
#[cfg(feature = "gpu-compute")]
mod capture;
mod panel;
pub struct ProjectPlugin;
impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Files>().add_systems(
            bevy_egui::EguiPrimaryContextPass,
            panel::show.run_if(in_state(engine_core::prelude::AppState::InGame)),
        );
    }
}
#[derive(Resource)]
pub struct Files {
    pub path: String,
    pub status: String,
    pub load: Option<Project>,
    pub save: Option<Pending>,
}
impl Default for Files {
    fn default() -> Self {
        Self {
            path: "soul-project.json".into(),
            status: String::new(),
            load: None,
            save: None,
        }
    }
}
pub struct Pending {
    pub project: Project,
    pub owners: Vec<(Entity, LeniaSimulation)>,
    pub path: std::path::PathBuf,
    pub started: std::time::Instant,
}
