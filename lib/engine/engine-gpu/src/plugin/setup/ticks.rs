//! Fixed-clock requests are counted once in the main world, independent of views.
use bevy::prelude::*;
use engine_core::{
    automata::{
        channel::{ChannelSettings, StepClock},
        lenia::LeniaSimulation,
    },
    prelude::AppState,
    systems::simulation::{FixedStepConfig, SimulationStep},
};

pub(in crate::plugin) fn request_ticks(
    mut events: EventReader<SimulationStep>,
    state: Res<State<AppState>>,
    config: Option<Res<FixedStepConfig>>,
    mut simulations: Query<(
        &mut LeniaSimulation,
        Option<(&ChannelSettings, &mut StepClock)>,
    )>,
) {
    let steps = events.read().count() as u64;
    if *state.get() != AppState::InGame {
        return;
    }
    // Base rate of SimulationStep; channels at the base rate pass through exactly.
    let base_hz = config.map_or(60, |c| (1.0 / c.dt.as_secs_f64()).round() as u32);
    for (mut simulation, pacing) in &mut simulations {
        if simulation.paused {
            continue;
        }
        simulation.target_tick += match pacing {
            Some((settings, mut clock)) => {
                clock.due(steps, settings.rate_hz.round() as u32, base_hz)
            }
            None => steps,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenia_fixed_ticks_respect_both_pause_layers() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<AppState>()
            .add_event::<SimulationStep>()
            .add_systems(Update, request_ticks);
        let owner = app.world_mut().spawn(LeniaSimulation::default()).id();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.update();
        for _ in 0..3 {
            app.world_mut().send_event(SimulationStep);
        }
        app.update();
        assert_eq!(
            app.world()
                .get::<LeniaSimulation>(owner)
                .unwrap()
                .target_tick,
            3
        );
        app.world_mut()
            .get_mut::<LeniaSimulation>(owner)
            .unwrap()
            .paused = true;
        app.world_mut().send_event(SimulationStep);
        app.update();
        assert_eq!(
            app.world()
                .get::<LeniaSimulation>(owner)
                .unwrap()
                .target_tick,
            3
        );
        app.world_mut()
            .get_mut::<LeniaSimulation>(owner)
            .unwrap()
            .paused = false;
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Paused);
        app.world_mut().send_event(SimulationStep);
        app.update();
        assert_eq!(
            app.world()
                .get::<LeniaSimulation>(owner)
                .unwrap()
                .target_tick,
            3
        );
    }

    #[test]
    fn paced_lenia_owner_requests_ticks_at_its_channel_rate() {
        use engine_core::automata::channel::{ChannelSettings, StepClock};
        use engine_core::systems::simulation::FixedStepConfig;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<AppState>()
            .insert_resource(FixedStepConfig::from_hz(60, 5))
            .add_event::<SimulationStep>()
            .add_systems(Update, request_ticks);
        let owner = app
            .world_mut()
            .spawn((
                LeniaSimulation::default(),
                ChannelSettings::new(10.0),
                StepClock::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.update();
        for _ in 0..60 {
            app.world_mut().send_event(SimulationStep);
        }
        app.update();
        let sim = app.world().get::<LeniaSimulation>(owner).unwrap();
        assert_eq!(sim.target_tick, 10);
    }
}
