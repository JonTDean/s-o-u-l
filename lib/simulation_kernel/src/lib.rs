//! Public kernel API.

use crate::core::{
    cell::{CellCtx, CellOutcome},
    dim::Dimensionality,
};

pub mod core;
pub mod grid;
pub mod stepper;
pub mod atlas;

/// Trait implemented by every user-defined automaton rule.
///
/// *Implementers must now specify the `D` alias explicitly*
/// (e.g. `type D = Dim;`) because associated-type defaults are
/// still unstable on the stable compiler toolchain.
pub trait AutomatonRule: Send + Sync + 'static {
    /// Simulation dimensionality (e.g. `Dim` for voxels).
    type D: Dimensionality;

    /// Computes the next state/memory for a cell.
    fn next_state<'a>(
        &self,
        ctx:    CellCtx<'a, Self::D>,
        params: &serde_json::Value,
    ) -> CellOutcome;
}
