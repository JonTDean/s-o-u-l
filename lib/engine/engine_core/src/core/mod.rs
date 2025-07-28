pub mod dim;
pub mod cell;
pub mod world;

pub use dim::*;
pub use cell::*;
pub use world::*;


pub trait AutomatonRule: Send + Sync + 'static {
    type D: Dim;

    fn next_state<'a>(
        &self,
        ctx: CellCtx<'a, Self::D>,
        params: &serde_json::Value,
    ) -> CellOutcome;
}