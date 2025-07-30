use bevy::prelude::*;
use engine_core::{
    events::AutomataCommand,
    prelude::RuleRegistry,
    world::World2D,
};

use super::wolfram_1d::{
    seed_rule30,
    seed_rule110,
    rules::{rule30::Rule30, rule110::Rule110},
};

/// Registers the Wolfram 1-D rules and seeds, and listens for
/// `AutomataCommand::SeedPattern` events so the user/UI can spawn them.
pub struct RegularAutomataPlugin;

impl Plugin for RegularAutomataPlugin {
    fn build(&self, app: &mut App) {
        /* 1 rule registry ------------------------------------------ */
        let mut reg = app
            .world_mut()
            .remove_resource::<RuleRegistry>()
            .unwrap_or_default();

        reg.register_with_seed("wolfram:rule30",  Rule30::boxed(),  seed_rule30);
        reg.register_with_seed("wolfram:rule110", Rule110::boxed(), seed_rule110);

        app.insert_resource(reg);

        /* 2 react to “seed” UI commands ---------------------------- */
        app.add_systems(
            Update,
            Self::on_seed_event.run_if(
                bevy::ecs::schedule::common_conditions::resource_exists::<World2D>,
            ),
        );
    }
}

impl RegularAutomataPlugin {
    /// When the UI asks to seed a pattern, spawn it inside the live
    /// `World2D` grid using the default seeder stored in `RuleRegistry`.
    fn on_seed_event(
        mut events:   EventReader<AutomataCommand>,
        world_opt:    Option<ResMut<World2D>>,
        registry:     Res<RuleRegistry>,
    ) {
        let Some(mut world) = world_opt else { return }; // wait until World2D exists

        for ev in events.read() {
            if let AutomataCommand::SeedPattern { id } = ev {
                registry.spawn_default(id, &mut world);
            }
        }
    }
}
