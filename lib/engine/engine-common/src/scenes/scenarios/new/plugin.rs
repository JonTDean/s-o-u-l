//! Plug-in and minimal draft buffer for the *New Scenario* flow.

use bevy::prelude::*;
use engine_core::prelude::AppState;

/// Draft metadata collected from the UI.
#[derive(Resource, Default, Clone)]
pub struct NewScenarioDraft {
    /// Scenario name typed by the user (may be empty until committed).
    pub name: String,
}

/// Registers `NewScenarioDraft` for the lifetime of `AppState::NewScenario`.
pub struct NewScenarioScenePlugin;

impl Plugin for NewScenarioScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::NewScenario),
            |mut cmd: Commands| cmd.insert_resource(NewScenarioDraft::default()),
        )
        .add_systems(
            OnExit(AppState::NewScenario),
            |mut cmd: Commands| cmd.remove_resource::<NewScenarioDraft>(),
        );
    }
}
