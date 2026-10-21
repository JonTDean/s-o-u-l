//! Public re-exports for **engine-gpu**
//!
//! Down-stream crates (engine-render, game code, …) should _only_ use
//! the surface re-exports from this file – everything else is private
//! implementation detail.

#![warn(missing_docs)]

pub use fixed_sim::FixedSimPlugin;
pub use plugin::{GpuAutomataComputePlugin, VoxelRenderStatus};
pub use render::compute::lenia::runtime::probe::{LeniaProbe, LeniaSnapshot};
pub use render::pipelines::GpuPipelineCache;
pub use types::AutomatonParams;

/* internal */
mod render;
use render::{compute, graph, pipelines};
mod fixed_sim;
pub mod plugin;
mod seed;
mod types;

#[cfg(all(
    feature = "mesh_shaders",
    any(
        windows,
        all(unix, not(target_os = "emscripten"), not(target_vendor = "apple"))
    )
))]
pub use render::ash;
