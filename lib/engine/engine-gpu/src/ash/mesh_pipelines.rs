//! Container for Vulkan graphics pipelines created by the mesh-shader backend.
//! 
//! Currently, this resource holds only a single pipeline (`mesh_pipeline`), but it can be 
//! extended in the future to manage multiple pipeline variants (e.g., for different render passes 
//! or debug pipelines like wireframes).
//! 
//! The `MeshPipelines` resource is `Send + Sync` (just wraps raw handles) and can be stored 
//! in Bevy’s ECS world. We rely on Bevy to automatically drop the Vulkan device (and thus all 
//! pipelines) at app exit, so we do not manually destroy the pipeline here.
//! 
//! # Safety 
//! The stored `vk::Pipeline` must not be used after the Vulkan device is dropped. It remains valid 
//! for the lifetime of the `AshContext`/`RenderDevice`. We do **not** manually destroy it; it will 
//! be cleaned up when the device is dropped by Bevy.

#![cfg(feature = "mesh_shaders")]

use ash::vk;
use bevy::prelude::*;

/// Resource containing handles to Vulkan pipelines for mesh shading.
#[derive(Resource, Clone, Copy)]
pub struct MeshPipelines {
    /// Vulkan pipeline handle for mesh shading (TASK+MESH stages).
    pub mesh_pipeline: vk::Pipeline,
}

impl MeshPipelines {
    /// Create a new `MeshPipelines` resource from an existing pipeline handle.
    /// 
    /// # Safety 
    /// The caller must guarantee that the provided `pipeline` remains valid for the 
    /// duration of the Bevy `App`. In practice, this means the pipeline is created on the same 
    /// `ash::Device` that will be destroyed only when the app shuts down (which is true if 
    /// created via our `AshContext` and not destroyed manually).
    pub const unsafe fn new(pipeline: vk::Pipeline) -> Self {
        MeshPipelines { mesh_pipeline: pipeline }
    }
}
