//! Render-world ownership and extraction for live Lenia simulations.
use bevy::{prelude::*, render::Extract};
use engine_core::automata::{GpuGridSlice, lenia::LeniaSimulation};
mod execute;
mod node;
pub(crate) mod probe;
pub(crate) mod state;
pub(crate) use node::LeniaStepNode;
pub(crate) use state::GpuSimulations;

#[derive(Resource, Default)]
pub(crate) struct LiveSimulations(pub Vec<(Entity, GpuGridSlice, LeniaSimulation)>);

pub(crate) fn extract(
    query: Extract<Query<(Entity, &GpuGridSlice, &LeniaSimulation)>>,
    mut live: ResMut<LiveSimulations>,
) {
    live.0 = query
        .iter()
        .map(|(entity, slice, sim)| (entity, slice.clone(), sim.clone()))
        .collect();
}
