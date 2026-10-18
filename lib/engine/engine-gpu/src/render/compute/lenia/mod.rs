//! Per-slice Lenia buffers and deterministic GPU updates.
mod buffers;
mod pipelines;
pub(crate) use buffers::{GpuParams, LeniaBuffers};
pub(crate) use pipelines::LeniaPipelines;
