//! Low-level Vulkan® bridge used only when the optional
//! `mesh_shaders` feature is enabled.
//
//! * Wraps the raw handles exposed by Bevy / wgpu.
//! * Does **not** create a second logical device – we piggy-back on
//!   the one already managed by wgpu so memory residency and queues
//!   stay coherent.
//!
//! ────────────────────────────────────────────────────────────────
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//

use ash::{vk, Device, Entry, Instance};
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use wgpu::hal::api::Vulkan as VkApi;

/// Shared Vulkan handles – lives in the **render world**.
#[derive(Resource)]
pub struct AshContext {
    pub entry:    Entry,
    pub instance: Instance,
    pub device:   Device,
    pub queue:    vk::Queue,
}

impl FromWorld for AshContext {
    fn from_world(world: &mut World) -> Self {
        let rd = world.resource::<RenderDevice>();

        // SAFETY: we only *borrow* raw handles from wgpu; lifetime is tied
        // to the Bevy device which outlives this resource.
        let (raw_inst, raw_dev, raw_q) = unsafe {
            rd.wgpu_device().as_hal::<VkApi, _, _>(|backend| {
                let queue = backend.raw_queue_group(0).queues[0];
                (backend.instance(), backend.raw_device(), queue)
            })
        }
        .expect("Vulkan backend required for `mesh_shaders` feature");

        // Portable loader (replaces deprecated `Entry::load()` pattern).
        let entry    = unsafe { Entry::linked() };
        let instance = unsafe { Instance::load(&entry, raw_inst.handle()) };
        let device   = unsafe { Device::load(&instance, raw_dev.handle()) };

        Self { entry, instance, device, queue: raw_q }
    }
}
