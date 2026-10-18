//! GPU execution: compute kernels, render-graph integration, and pipeline caching.
#[cfg(feature = "mesh_shaders")]
pub mod ash;
pub(crate) mod compute;
pub(crate) mod graph;
pub(crate) mod pipelines;
