//! GPU compute back-end for cellular automata.
//!
//! This crate implements the render-graph node and WGSL pipelines that
//! execute automata updates entirely on the GPU.  It exposes a single Bevy
//! [`Plugin`] (`GpuAutomataComputePlugin`) that allocates shared textures and
//! dispatches compute shaders each frame.  Other crates only need to add this
//! plugin to gain GPU acceleration.
//!
//! # Re-exports
//! The [`GpuAutomataComputePlugin`] and [`AutomatonParams`] types are
//! re-exported at the crate root for convenience.
//!
pub use self::{plugin::GpuAutomataComputePlugin, types::AutomatonParams};

mod types;
mod pipelines;
mod graph;
mod plugin;    // <‑ full implementation below
