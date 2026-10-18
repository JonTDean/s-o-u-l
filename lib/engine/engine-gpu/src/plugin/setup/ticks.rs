//! Fixed-clock requests are counted once in the main world, independent of views.
use bevy::prelude::*;
use engine_core::{
    automata::lenia::LeniaSimulation, prelude::AppState, systems::simulation::SimulationStep,
};

pub(in crate::plugin) fn request_ticks(
    mut events: EventReader<SimulationStep>,
    state: Res<State<AppState>>,
    mut simulations: Query<&mut LeniaSimulation>,
) {
    let steps = events.read().count() as u64;
    if *state.get() != AppState::InGame {
        return;
    }
    for mut simulation in &mut simulations {
        if !simulation.paused {
            simulation.target_tick += steps;
        }
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
}
