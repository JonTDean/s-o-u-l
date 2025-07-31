//! Core simulation primitives shared by all automata types.
//!
//! `simulation_kernel` defines the grid storage back-ends and stepping
//! algorithms used by both the CPU and GPU implementations.  It also exposes
//! the [`AutomatonRule`] trait which all rule implementations must fulfil.

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