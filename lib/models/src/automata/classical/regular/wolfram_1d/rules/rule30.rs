use bevy::prelude::Resource;
use simulation_kernel::{core::{
    cell::{CellCtx, CellOutcome, CellState}, dim::Dim,
}, AutomatonRule};
use serde_json::Value;

/// Hard‑coded rule table for Wolfram 30.
const RULE_30: [u8; 8] = [0,1,1,1,1,0,0,0];

#[derive(Clone, Resource)]
pub struct Rule30;


impl Rule30 {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim> + Send + Sync> {
        std::sync::Arc::new(Self)
    }
}

impl AutomatonRule for Rule30 {
    type D = Dim;
    fn next_state(&self, ctx: CellCtx<Self::D>, _params: &Value) -> CellOutcome {
        // Interpret 3 neighbors in a horizontal line: left (W), center (self), right (E).
        let l = matches!(ctx.neighbourhood[3], CellState::Alive(_)) as u8;
        let c = matches!(ctx.self_state,       CellState::Alive(_)) as u8; // centre
        let r = matches!(ctx.neighbourhood[5], CellState::Alive(_)) as u8;
        let idx = (l << 2) | (c << 1) | r;
        let next_state = if RULE_30[idx as usize] == 1 {
            CellState::Alive(255)
        } else {
            CellState::Dead
        };
        if next_state == ctx.self_state {
            CellOutcome::Unchanged
        } else {
            CellOutcome::Next { state: next_state, memory: ctx.memory.clone() }
        }
    }
}

