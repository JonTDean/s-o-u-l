//! Life-like cellular automata (Conway's Life and a multi-state variant).

use bevy::prelude::Resource;
use engine_core::core::{
    AutomatonRule, CellCtx, CellOutcome,
    cell::CellState, dim::Dim2,
};
use serde_json::{Value, Number};

#[derive(Clone, Resource)]
pub struct ConwayLife;

impl ConwayLife {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim2> + Send + Sync> {
        std::sync::Arc::new(Self)
    }
}

impl AutomatonRule for ConwayLife {
    type D = Dim2;
    fn next_state(&self, ctx: CellCtx<Self::D>, _params: &Value) -> CellOutcome {
        // Count alive neighbors (8 neighbors in Moore neighborhood).
        let alive_neighbors = ctx.neighbourhood
            .iter()
            .filter(|&&state| matches!(state, CellState::Alive(_)))
            .count();
        // Determine if current cell was alive (store as 1 in memory, 0 if dead).
        let was_alive = ctx.memory.as_u64().unwrap_or(0) == 1;
        // Conway's Life rules:
        let next_state = if was_alive {
            // If currently alive:
            if alive_neighbors < 2 || alive_neighbors > 3 {
                // Dies by under/overpopulation.
                CellState::Dead
            } else {
                // Continues living.
                CellState::Alive(255)
            }
        } else {
            // If currently dead:
            if alive_neighbors == 3 {
                // Birth by reproduction.
                CellState::Alive(255)
            } else {
                CellState::Dead
            }
        };
        // Determine outcome (only produce Next if state or memory changes).
        if was_alive && matches!(next_state, CellState::Alive(_))
            || !was_alive && matches!(next_state, CellState::Dead)
        {
            // State remains the same (alive->alive or dead->dead)
            CellOutcome::Unchanged
        } else {
            // State changes (alive->dead or dead->alive): update state and memory.
            let new_memory = if matches!(next_state, CellState::Alive(_)) {
                Value::Number(Number::from(1))
            } else {
                Value::Number(Number::from(0))
            };
            CellOutcome::Next { state: next_state, memory: new_memory }
        }
    }
}