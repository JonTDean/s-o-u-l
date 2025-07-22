//! Naïve single‑threaded world stepper.
//! • Picks the first rule in [`RuleRegistry`].
//! • Calculates all updates in a read‑only pass.
//! • Applies updates in a second mutable pass.

use std::marker::PhantomData;

use bevy::prelude::*;
use engine_core::{
    core::{CellCtx, CellOutcome, dim::Dim2},
    core::world::World2D,
    engine::grid::GridBackend,
};
use serde_json::Value;
use crate::registry::RuleRegistry;

/// Parameters sent to every rule call (empty for now).
const EMPTY_PARAMS: Value = Value::Null;

/* ──────────────────────────────────────────────────────────────────────── */

pub struct WorldStepperPlugin;
impl Plugin for WorldStepperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RuleRegistry>()
            .add_systems(
                Update,
                step_world
                    .in_set(engine_core::schedule::MainSet::Logic)
                    // run only while the game is actually running…
                    .run_if(in_state(engine_core::state::AppState::InGame))
                    // …and only once World2D has been created
                    .run_if(resource_exists::<World2D>),
            );
    }
}
/* ──────────────────────────────────────────────────────────────────────── */

fn step_world(
    mut world: ResMut<World2D>,
    rules:     Res<RuleRegistry>,
) {
    /* 1 ── pick *any* active rule (first entry for now) */
    let Some(rule) = rules.ids().next().and_then(|id| rules.get(id)) else { return };

    /* 2 ── READ‑ONLY pass: clone cells & gather updates */
    let (updates, size) = {
        let GridBackend::Dense(grid) = &world.backend else { return };
        let size = grid.size;
        let prev_cells = grid.cells.clone();

        let mut ups = Vec::new();

        for (idx, cell) in prev_cells.iter().enumerate() {
            let coord = IVec2::new(
                (idx as u32 % size.x) as i32,
                (idx as u32 / size.x) as i32,
            );

            let neighbourhood = world.neighbourhood(coord);

            let ctx = CellCtx::<Dim2> {
                self_coord: coord,
                self_state: cell.state,            
                neighbourhood: &neighbourhood,
                memory: &cell.memory,
                _marker: PhantomData::<Dim2>,              //  (or `PhantomData::<Dim2>`)
            };

            if let CellOutcome::Next { state, memory } =
                rule.next_state(ctx, &EMPTY_PARAMS)
            {
                ups.push((idx, state, memory));
            }
        }

        (ups, size)
    };

    /* 3 ── WRITE pass: apply queued updates mutably */
    if let GridBackend::Dense(grid) = &mut world.backend {
        for (idx, state, memory) in updates {
            let tgt = grid.cells.get_mut(idx).expect("index in‑bounds");
            tgt.state  = state;
            tgt.memory = memory;
        }
    }
}
