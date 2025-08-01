//! Public re-exports for **engine-gpu**
//!
//! Down-stream crates (engine-render, game code, …) should _only_ use
//! the surface re-exports from this file – everything else is private
//! implementation detail.

// Public re-exports for engine-gpu.
pub use plugin::GpuAutomataComputePlugin;
pub use types::AutomatonParams;

/* Internal modules */
mod types;
mod pipelines;
mod graph;
mod plugin;
mod compute;
mod seed;
#[cfg(feature = "mesh_shaders")]
pub mod ash;
