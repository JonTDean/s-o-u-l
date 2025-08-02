#![allow(unused_imports)]
use std::time::Duration;

use bevy::{
    prelude::PluginGroup,
    app::{App, Update},
    time::{Time, TimePlugin},
    MinimalPlugins,
};
use bevy_ecs::event::Events;
use engine_core::systems::simulation::{
    accumulate_and_step, FixedStepConfig, SimAccumulator, SimulationStep,
};

#[test]
fn accumulator_emits_correct_steps() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins.build().disable::<TimePlugin>()) // now resolves
        .init_resource::<Time>()                       // canonical clock
        .insert_resource(FixedStepConfig::from_hz(60, 5))
        .insert_resource(SimAccumulator::default())
        .add_event::<SimulationStep>()
        .add_systems(Update, accumulate_and_step);

    let dt   = Duration::from_millis(8);               // 4 × 8 ms = 32 ms
    let mut steps = 0;

    for _ in 0..4 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(dt);                           // deterministic tick
        app.update();

        steps += app
            .world_mut()
            .resource_mut::<Events<SimulationStep>>()
            .drain()
            .count();
    }

    assert_eq!(steps, 1);                              // 32 ms ⇒ 1 fixed step
}
