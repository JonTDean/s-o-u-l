use crate::core::{cell::{CellCtx, CellOutcome}, dim::Dim};

pub mod core;
pub mod grid;
pub mod stepper;
pub mod atlas;


pub trait AutomatonRule: Send + Sync + 'static {
    type D: Dim;

    fn next_state<'a>(
        &self,
        ctx: CellCtx<'a, Self::D>,
        params: &serde_json::Value,
    ) -> CellOutcome;
}