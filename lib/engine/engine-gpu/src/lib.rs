//! Public re-exports for **engine-gpu**
//!
//! Down-stream crates (engine-render, game code, …) should _only_ use
//! the surface re-exports from this file – everything else is private
//! implementation detail.

#![warn(missing_docs)]

pub use plugin::GpuAutomataComputePlugin;
pub use fixed_sim::FixedSimPlugin;
pub use types::AutomatonParams;

/* internal */
mod compute;
mod graph;
mod pipelines;
pub mod plugin;
mod seed;
mod types;
mod fixed_sim;

#[cfg(feature = "mesh_shaders")]
pub mod ash;
