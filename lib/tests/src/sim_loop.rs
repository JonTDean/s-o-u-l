//! Fixed-clock simulation behavior, independent of wall-clock scheduling jitter.
use bevy::{prelude::*, time::Fixed};
use bevy_ecs::event::Events;
use engine_core::systems::simulation::{
    FixedStepConfig, SimAccumulator, SimulationStep, accumulate_and_step,
};
use std::time::Duration;

fn test_app(max_steps: u8) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(FixedStepConfig::from_hz(60, max_steps))
        .init_resource::<SimAccumulator>()
        .add_event::<SimulationStep>()
        .add_systems(FixedUpdate, accumulate_and_step);
    app
}

fn advance(app: &mut App, milliseconds: u64) -> usize {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_millis(milliseconds));
    app.world_mut().run_schedule(FixedUpdate);
    app.world_mut()
        .resource_mut::<Events<SimulationStep>>()
        .drain()
        .count()
}

#[test]
fn accumulator_emits_correct_steps() {
    let mut app = test_app(5);
    let total: usize = (0..4).map(|_| advance(&mut app, 8)).sum();
    assert_eq!(total, 1); // 32 ms contains one 60 Hz tick, retaining its remainder.
    assert!(app.world().resource::<SimAccumulator>().accum > 0.0);
}

#[test]
fn spiral_guard_caps_work_and_discards_excess_lag() {
    let mut app = test_app(2);
    assert_eq!(advance(&mut app, 1_000), 2);
    assert_eq!(app.world().resource::<SimAccumulator>().accum, 0.0);
    assert_eq!(advance(&mut app, 8), 0);
}
