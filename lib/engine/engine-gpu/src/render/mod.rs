//! GPU execution: compute kernels, render-graph integration, and pipeline caching.
#[cfg(all(
    feature = "mesh_shaders",
    any(
        windows,
        all(unix, not(target_os = "emscripten"), not(target_vendor = "apple"))
    )
))]
pub mod ash;
pub(crate) mod compute;
pub(crate) mod graph;
pub(crate) mod pipelines;
