//! computational_intelligence/bridges/world_stepper.rs
//! Parallel version:                                                     │
//! • `par_bridge()` → coarse‑grain parallelism across automata           │
//! • Rayon `par_iter_mut()` inside `step_dense_dyn_parallel()` for grids │

use bevy::{
    prelude::IntoScheduleConfigs,
    app::{App, Plugin, Update}, 
    ecs::system::ResMut, 
    state::condition::in_state,
};
use crate::systems::{registry::AutomataRegistry, schedule, state::AppState};
use serde_json::Value;
use rayon::prelude::*;
use simulation_kernel::{
    grid::GridBackend,
    stepper::{
        dense,
        sparse
    },
    
};


/* ------------------------------------------------- */

const EMPTY: Value = Value::Null;

pub struct WorldStepperPlugin;
impl Plugin for WorldStepperPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            step_every_automaton_parallel
                .in_set(schedule::MainSet::Logic)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/* ------------------------------------------------- */

fn step_every_automaton_parallel(mut reg: ResMut<AutomataRegistry>) {
    // 1 ── coarse‑grain: each automaton on its own Rayon task
    reg.iter_mut()
       .par_bridge()                      // converts iterator → ParallelIterator
       .for_each(|auto| {
           match &mut auto.grid {
               GridBackend::Dense(g)  => dense::parallel::step_dense_dyn_parallel(g, &*auto.rule, &EMPTY),
               GridBackend::Sparse(s) => sparse::parallel::step_sparse_dyn_parallel(s, &*auto.rule, &EMPTY),
           }
       });
}
