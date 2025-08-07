use bevy::prelude::*;
use engine_core::systems::{
    schedule::MainSet,
    simulation::{accumulate_and_step, FixedStepConfig, SimAccumulator},
};

/// 60 Hz deterministic stepper with a 5-tick spiral-guard.
pub struct FixedSimPlugin;
impl Plugin for FixedSimPlugin {
    fn build(&self, app: &mut App) {
        app
            // 60 Hz → dt = 16 ⅔ ms
            .insert_resource(FixedStepConfig::from_hz(60, 5))
            .insert_resource(SimAccumulator::default())
            // run first inside the FixedUpdate tier
            .add_systems(
                FixedUpdate,
                accumulate_and_step.in_set(MainSet::Input),
            );
    }
}
