
use bevy::prelude::*;
use serde_json::Value;
use engine_core::engine::stepper::plugin::StepperPlugin;
use crate::registry::RuleRegistry;
use super::life::ConwayLife;

/// Aggregates every dynamical family of automata.
pub struct DynamicalAutomataPlugin;
impl Plugin for DynamicalAutomataPlugin {
    fn build(&self, app: &mut App) {
        // Register continuous/discrete dynamical automata rules (Conway, Dean's Life, etc.)
        let mut reg = app.world_mut()
                         .remove_resource::<RuleRegistry>()
                         .unwrap_or_default();
        reg.register("life:conway", ConwayLife::boxed());
        app.insert_resource(reg);

        // Spawn steppers for Life rules.
        app.add_plugins(StepperPlugin::<ConwayLife> {
            rule:   ConwayLife,
            params: Value::Null,
        });

        // Note: Lenia and other continuous automata are not yet integrated with the stepper.
    }
}