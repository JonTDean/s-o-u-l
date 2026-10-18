//! Per-slice Lenia buffers and deterministic GPU updates.
mod buffers;
mod dispatch;
mod pipelines;
pub(crate) use buffers::{GpuParams, LeniaBuffers};
pub(crate) use pipelines::LeniaPipelines;

#[cfg(test)]
mod tests;
