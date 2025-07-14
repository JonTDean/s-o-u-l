mod ca_engine;

use bevy::prelude::*;
use crate::ca_engine::{
    core::{
        Cell, 
        World2D, 
        Dim2, 
        CellState, 
        AutomatonRule, 
        CellCtx, 
        CellOutcome
    },
    grid::{GridBackend, DenseGrid},
    renderer::Renderer2DPlugin,
    stepper::StepperPlugin,
};


fn main() {
    let size = UVec2::new(64, 64);
    let cells = vec![Cell::default(); (size.x * size.y) as usize];

    App::new()
        .insert_resource(World2D {
            backend: GridBackend::Dense(DenseGrid { cells, size }),
            cell_size: 10.0,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins((
            Renderer2DPlugin,
            StepperPlugin {
                rule:   ConwayRule,
                params: serde_json::json!({}),
            },
        ))
        .run();
}

/// -----------------------------------------------------------------
/// Example Conway rule (B3/S23) just to prove the pipeline.
/// Put real rule-sets under `intelligences/automata`.
/// -----------------------------------------------------------------
#[derive(Resource, Clone)]
struct ConwayRule;

impl AutomatonRule for ConwayRule {
    type D = Dim2;

    fn next_state(
        &self,
        ctx: CellCtx<Self::D>,
        _params: &serde_json::Value,
    ) -> CellOutcome {
        let live_n = ctx.neighbourhood
                        .iter()
                        .filter(|&&s| matches!(s, CellState::Alive(_)))
                        .count() as u8;

        match ctx.neighbourhood[4] { // centre slot == self?  we passed without self; treat separately
            _ => {}
        }

        let currently_alive =
            matches!(ctx.neighbourhood[4], CellState::Alive(_)); // shortcut

        let next_alive = match (currently_alive, live_n) {
            (true, 2 | 3) => true,
            (false, 3)    => true,
            _             => false,
        };

        if next_alive != currently_alive {
            CellOutcome::Next {
                state:  if next_alive { CellState::Alive(1) } else { CellState::Dead },
                memory: ctx.memory.clone(),
            }
        } else {
            CellOutcome::Unchanged
        }
    }
}
