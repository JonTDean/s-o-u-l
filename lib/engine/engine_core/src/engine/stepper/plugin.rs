//! engine/stepper/plugin.rs
use std::time::Duration;

use bevy::{
    app::{App, FixedUpdate, Plugin},
    ecs::{prelude::{Res, ResMut, Resource}, schedule::{common_conditions::resource_exists, IntoScheduleConfigs}},
    state::condition::in_state, time::common_conditions::on_timer,  
};
use crate::{
    core::{AutomatonRule, Dim2, World2D},
    engine::{grid::GridBackend, stepper::{dense::step_dense, sparse::step_sparse, RuleParams}}, state::AppState,
};

pub struct StepperPlugin<R: Resource + Clone> {
    pub rule:   R,
    pub params: serde_json::Value,
}

impl<R: AutomatonRule<D = Dim2> + Clone + Resource> Plugin for StepperPlugin<R> {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.rule.clone())
           .insert_resource(RuleParams(self.params.clone()))
            .add_systems(
                FixedUpdate,
                step_world::<R>
                    .run_if(resource_exists::<World2D>)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_timer(Duration::from_secs_f64(1.0 / 30.0))) // 30 Hz CA tick
            );
    }
}

/// Steps the world using the provided rule and parameters.
fn step_world<R: AutomatonRule<D = Dim2> + Resource>(
    mut world: ResMut<World2D>,
    rule:      Res<R>,
    params:    Res<RuleParams>,
) {
    match &mut world.backend {
        GridBackend::Dense(g)  => step_dense(g, &*rule, &params.0),
        GridBackend::Sparse(g) => step_sparse(g, &*rule, &params.0),
    }
}
