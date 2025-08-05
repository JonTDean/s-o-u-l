//! Aggregator plug-in that bundles the *New* and *Load* scenario scenes.

use bevy::prelude::*;

use super::{
    load::plugin::LoadScenarioScenePlugin,
    new::plugin::NewScenarioScenePlugin,
};

/// Registers both scenario creation flows.
///
/// This module contains **no** logic of its own so remains perfectly safe in
/// headless builds.
pub struct ScenariosPlugin;

impl Plugin for ScenariosPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NewScenarioScenePlugin,
            LoadScenarioScenePlugin,
        ));
    }
}
