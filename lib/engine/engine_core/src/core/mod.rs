pub mod dim;
pub mod cell;
pub mod world;

use std::marker::PhantomData;

use bevy::math::IVec2;
pub use dim::*;
pub use cell::*;
pub use world::*;

/// Context provided to every rule invocation.
pub struct CellCtx<'a, D: Dim> {
    pub self_coord:   IVec2,
    pub self_state:   CellState,
    pub neighbourhood: &'a [CellState; 8],
    pub memory:       &'a CellMemory,        // <- align with `cell.rs`
    pub _marker:          PhantomData<D>,        // <- silences the lint ✔
}

pub enum CellOutcome {
    Unchanged,
    Next { state: cell::CellState, memory: cell::CellMemory },
}

pub trait AutomatonRule: Send + Sync + 'static {
    type D: Dim;

    fn next_state<'a>(
        &self,
        ctx: CellCtx<'a, Self::D>,
        params: &serde_json::Value,
    ) -> CellOutcome;
}