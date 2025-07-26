use bevy::prelude::*;
use serde_json::Value;
use engine_core::{engine::stepper::plugin::StepperPlugin, events::AutomataCommand};
use engine_core::core::World2D;
 
use crate::registry::RuleRegistry;
use super::wolfram_1d::{
    seed_rule30, 
    seed_rule110, 
    rules::{
        rule30::Rule30, 
        rule110::Rule110
    }
};
pub struct RegularAutomataPlugin;

impl Plugin for RegularAutomataPlugin {
     fn build(&self, app: &mut App) {
        // Register rules in the global registry, including default seed patterns
        let mut reg = app.world_mut().remove_resource::<RuleRegistry>().unwrap_or_default();
        reg.register_with_seed("wolfram:rule30", Rule30::boxed(), seed_rule30);
        reg.register_with_seed("wolfram:rule110", Rule110::boxed(), seed_rule110);
        app.insert_resource(reg);
 
        // Add CPU stepper systems for each rule
        app.add_plugins((
            StepperPlugin::<Rule30> { rule: Rule30, params: Value::Null },
            StepperPlugin::<Rule110> { rule: Rule110, params: Value::Null },
        ))
        .add_systems(
        Update, 
        Self::on_seed_event
            .run_if(bevy::ecs::schedule::common_conditions::resource_exists::<World2D>)
        );
     }
 }

impl RegularAutomataPlugin {
    fn on_seed_event(
        mut events: EventReader<AutomataCommand>,
        world_opt:   Option<ResMut<World2D>>,
        registry: Res<RuleRegistry>,
    ) {
        let Some(mut world) = world_opt else { return };   // skip until ready
        for ev in events.read() {
            if let AutomataCommand::SeedPattern { id } = ev {
                registry.spawn_default(id, &mut world);
            }
        }
    }
}