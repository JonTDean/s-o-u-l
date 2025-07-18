use bevy::prelude::*;
use crate::intelligence_engine::{core::*, grid::*};

mod dense;
mod sparse;

pub use dense::step_dense;
pub use sparse::step_sparse;

#[derive(Resource, Clone)]
pub struct RuleParams(pub serde_json::Value);

pub struct StepperPlugin<R: AutomatonRule<D = Dim2> + Clone + Resource> {
    pub rule:   R,
    pub params: serde_json::Value,
}

impl<R: AutomatonRule<D = Dim2> + Clone + Resource> Plugin for StepperPlugin<R> {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.rule.clone())
           .insert_resource(RuleParams(self.params.clone()))
           .add_systems(FixedUpdate, step_world::<R>);
    }
}

fn step_world<R: AutomatonRule<D = Dim2> + Resource>(
    mut world: ResMut<World2D>,
    rule:       Res<R>,
    params:     Res<RuleParams>,
) {
    match &mut world.backend {
        GridBackend::Dense(g)  => step_dense(g, &*rule, &params.0),
        GridBackend::Sparse(g) => step_sparse(g, &*rule, &params.0),
    }
}