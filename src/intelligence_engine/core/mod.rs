pub mod dim;
pub mod cell;
pub mod world;

pub use dim::*;
pub use cell::*;
pub use world::*;

/// Context provided to every rule invocation.
pub struct CellCtx<'a, D: dim::Dim> {
    pub self_coord: D::Coord,
    pub neighbourhood: &'a [cell::CellState],
    pub memory: &'a cell::CellMemory,
}

pub enum CellOutcome {
    Unchanged,
    Next { state: cell::CellState, memory: cell::CellMemory },
}

pub trait AutomatonRule: Send + Sync + 'static {
    type D: dim::Dim;

    fn next_state(
        &self,
        ctx: CellCtx<Self::D>,
        params: &serde_json::Value,
    ) -> CellOutcome;
}