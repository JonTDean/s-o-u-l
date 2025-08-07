//! Vulkan-side integration for the mesh-shader fast path.
//! 
//! This module leverages **Ash** (Vulkan bindings) to issue low-level Vulkan commands 
//! on Bevy’s existing `wgpu` device. It wraps raw Vulkan handles from `wgpu`’s 
//! internal Vulkan backend, allowing use of Vulkan-specific features (like 
//! `VK_EXT_mesh_shader`) that are not directly exposed by wgpu.
//! 
//! Key responsibilities of this module:
//! - Initialize an `AshContext` resource holding Vulkan `Entry`, `Instance`, and `Device` handles 
//!   cloned from the `wgpu` device (so no second logical device is created).
//! - Create a mesh shader graphics pipeline (`MeshPipeline`) with TASK+MESH stages via Ash, 
//!   using WGSL shaders compiled to SPIR-V at runtime (through Naga).
//! - Provide a `MeshShaderNode` that injects Vulkan commands (pipeline barriers, draw calls) 
//!   into Bevy’s render graph when mesh shaders are supported.
//! 
//! This module is compiled and included only when the `mesh_shaders` Cargo feature is enabled 
//! **and** the app is running on the Vulkan backend (since other backends do not support mesh shaders).
//! 
//! ────────────────────────────────────────────────────────────────────
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

#![cfg(feature = "mesh_shaders")]

use ash::{Device, Entry, Instance, vk};
use bevy::{prelude::*, render::renderer::RenderDevice};
use wgpu::hal::api::Vulkan as VkApi;

/* Sub-modules */
pub mod mesh_pipeline;
pub mod mesh_pipelines;
pub mod mesh_shader_node;

/// Shared Vulkan (Ash) context cloned from Bevy’s `wgpu` device.
/// 
/// This resource provides low-level Vulkan handles needed to record and submit 
/// raw Vulkan commands (e.g., `vkCmdDrawMeshTasksEXT`). Cloning the handles from 
/// `wgpu` is very cheap (pointer copies) and does not affect the lifecycle 
/// of the underlying Vulkan objects managed by wgpu.
#[derive(Resource, Clone)]
pub struct AshContext {
    /// Vulkan entry point (dynamic library) handle.
    pub entry: Entry,
    /// Vulkan instance handle (application and driver connection).
    pub instance: Instance,
    /// Logical Vulkan device handle (wrapping the GPU device).
    pub device: Device,
    /// Graphics queue handle for command submission.
    pub queue: vk::Queue,
    /// Command pool for allocating command buffers on the graphics queue.
    pub command_pool: vk::CommandPool,
}

impl FromWorld for AshContext {
    fn from_world(world: &mut World) -> Self {
        // Access Bevy's RenderDevice (which owns the wgpu::Device)
        let render_device = world.resource::<RenderDevice>();
        // Use wgpu-hal (unsafe) to extract Ash handles from the wgpu device.
        let (entry, instance, device, queue, queue_family_index) = unsafe {
            render_device.wgpu_device().as_hal::<VkApi, _, _>(|hal_device_opt| {
                let hal_device = hal_device_opt.expect("Vulkan backend not active");
                // Get Vulkan handles from the HAL device (Arc<DeviceShared>)
                let entry    = hal_device.shared_instance().entry().clone();
                let instance = hal_device.shared_instance().raw_instance().clone();
                let device   = hal_device.raw_device().clone();
                let queue    = hal_device.raw_queue().clone();
                let family   = hal_device.queue_family_index();
                (entry, instance, device, queue, family)
            })
        };
        // Create a command pool for allocating command buffers on the graphics queue.
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe {
            device.create_command_pool(&pool_info, None)
                  .expect("Failed to create Vulkan command pool")
        };
        Self { entry, instance, device, queue, command_pool }
    }
}
