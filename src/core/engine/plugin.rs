use bevy::prelude::*;
use crate::app::schedule::MainSet;
use super::systems::step_world;

/// Core simulation plugin.
///
/// *In a later commit* this will:
///   * spawn grid entities/resources
///   * register rule-set sub-plugins (Type1 / Type2 / …)
///   * add fixed-timestep stepping
pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, step_world.in_set(MainSet::Logic));
    }
}
