//! Public re‑exports ────────────────────────────────────────────────────
pub use self::{plugin::GpuAutomataComputePlugin, types::AutomatonParams};

mod types;
mod pipelines;
mod graph;
mod plugin;    // <‑ full implementation below
