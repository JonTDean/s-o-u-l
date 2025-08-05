//! Plug-in that enumerates on-disk scenario saves and exposes the list as a
//! Bevy `Resource`.  The UI layer turns the manifest into a scroll-able
//! button list.

use bevy::prelude::*;
use dirs_next::data_dir;
use engine_core::prelude::AppState;
use std::path::PathBuf;

/// Immutable manifest of discovered save files.
#[derive(Resource, Clone, Debug)]
pub struct ScenarioManifest {
    /// Absolute paths to `*.soul.save` scenario files.
    pub files: Vec<PathBuf>,
}

impl Default for ScenarioManifest {
    fn default() -> Self { Self { files: Vec::new() } }
}

/// Recursively enumerates `*.soul.save` files inside the user data directory.
fn scan_manifest() -> Vec<PathBuf> {
    if let Some(base) = data_dir()
        .map(|d| d.join("phanestead").join("saves"))
    {
        std::fs::read_dir(base)
            .map(|iter| {
                iter.flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |e| e == "soul.save"))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// Registers `ScenarioManifest` for the lifetime of `AppState::LoadScenario`.
pub struct LoadScenarioScenePlugin;

impl Plugin for LoadScenarioScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::LoadScenario),
            |mut cmd: Commands| {
                cmd.insert_resource(ScenarioManifest { files: scan_manifest() });
            },
        )
        .add_systems(
            OnExit(AppState::LoadScenario),
            |mut cmd: Commands| cmd.remove_resource::<ScenarioManifest>(),
        );
    }
}
