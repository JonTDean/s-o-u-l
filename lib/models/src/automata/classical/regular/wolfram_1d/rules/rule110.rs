use bevy::prelude::Resource;
use simulation_kernel::{core::{
    cell::{CellCtx, CellOutcome, CellState}, dim::Dim2,
}, AutomatonRule};
use serde_json::Value;

/// Hard‑coded rule table for Wolfram 110.
const RULE_110: [u8; 8] = [0,1,1,0,1,1,1,0];

#[derive(Clone, Resource)]
pub struct Rule110;

impl Rule110 {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim2> + Send + Sync> {
        std::sync::Arc::new(Self)
    }
}

impl AutomatonRule for Rule110 {
    type D = Dim2;
    fn next_state(&self, ctx: CellCtx<Self::D>, _params: &Value) -> CellOutcome {
        // Similar neighborhood interpretation as Rule30.
        let l = matches!(ctx.neighbourhood[3], CellState::Alive(_)) as u8;
        let c = matches!(ctx.self_state,       CellState::Alive(_)) as u8; // centre
        let r = matches!(ctx.neighbourhood[5], CellState::Alive(_)) as u8;
        let idx = (l << 2) | (c << 1) | r;
        let next_state = if RULE_110[idx as usize] == 1 {
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
