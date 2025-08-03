//! Low-level Vulkan® bridge (compiled only when the `mesh_shaders` feature is on).
//!
//! We **borrow** the Vulkan handles that `wgpu` already owns so experimental
//! mesh-shader paths can issue raw calls through Ash without creating a second
//! logical device. **No Vulkan resources are created or destroyed here.**
//!
//! # Safety contract
//! * The game must run with the Vulkan backend whenever `mesh_shaders` is
//!   enabled (enforced in `build.rs`).
//! * `wgpu` keeps its raw handles alive for the entire process lifetime, so
//!   cloned Ash dispatch tables remain valid indefinitely.
//!
//! `AshContext` is `Send + Sync` because the wrapped Ash structs are immutable,
//! thread-safe dispatch tables that never touch ownership of the underlying
//! Vulkan objects.

#![cfg(feature = "mesh_shaders")]

use ash::{Device, Entry, Instance};
use bevy::{prelude::*, render::renderer::RenderDevice};
use wgpu::hal::api::Vulkan as VkApi;

/// Cheaply-cloned Ash wrappers around `wgpu`’s Vulkan objects.
#[derive(Resource, Clone)]
pub struct AshContext {
    /// Global Vulkan loader cloned from `wgpu`’s internals.
    pub entry: Entry,
    /// Dispatch table for the pre-existing `vk::Instance`.
    pub instance: Instance,
    /// Dispatch table for the pre-existing logical `vk::Device`.
    pub device: Device,
}

impl FromWorld for AshContext {
    /// Extract raw Vulkan handles from Bevy’s [`RenderDevice`] and wrap them
    /// in Ash dispatch tables that can be shared freely across systems.
    ///
    /// # Panics
    /// If the engine was launched without the Vulkan backend.
    fn from_world(world: &mut World) -> Self {
        // SAFETY: Completed upon enabling the `mesh_shaders` feature – the build
        // is Vulkan-only, so the HAL’s unsafe cast is sound.
        let rd = world.resource::<RenderDevice>();

        let (entry, instance, device) = unsafe {
            rd.wgpu_device().as_hal::<VkApi, _, _>(|opt_dev| {
                let dev = opt_dev.expect("Vulkan backend not active");
                let shared = dev.shared_instance();
                (
                    shared.entry().clone(),        // ash::Entry
                    shared.raw_instance().clone(), // ash::Instance
                    dev.raw_device().clone(),      // ash::Device
                )
            })
        };

        Self { entry, instance, device }
    }
}
