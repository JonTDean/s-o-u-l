//! Render graph node that issues a draw call using the mesh shader pipeline.
//! 
//! This node runs each frame during the render graph execution, but only if mesh shaders 
//! are supported by the GPU and the `mesh_shaders` feature is enabled (otherwise the whole module is disabled).
//! 
//! The node performs the following in its `run` method:
//! 1. Retrieves the required resources: `AshContext` (for Vulkan device/queue), `MeshletBuffers` (buffers produced by the compute step), and `MeshPipelines` (our mesh shader pipeline).
//! 2. Obtains the current viewport size from the primary window to set the dynamic viewport and scissor.
//! 3. Acquires the underlying Vulkan command buffer from Bevy’s `RenderContext` command encoder (using wgpu-hal) so we can record raw Vulkan commands.
//! 4. Issues a memory barrier to ensure the compute shader output (meshlet buffers) is visible to the GPU for drawing.
//! 5. Binds the mesh shader pipeline and vertex buffers, then issues an indirect draw call (`vkCmdDrawMeshTasksEXT` via `cmd_draw_indirect` on the indirect buffer).
//! 
//! This node is stateless (`Send + Sync`) and can run on any render thread. It does not store any frame-specific data internally.
//! 
//! # Note
//! The `MeshletBuffers` resource is produced by the compute stage (for example, via a compute node that runs the Dual Contouring algorithm). It contains the GPU buffers for vertices and the indirect draw parameters. These buffers must be created with usage flags that allow compute write and vertex/indirect read.
//! 
//! ───────────────────────────────────────────────────────────────────
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//! Render‑graph node that records one mesh‑shader indirect draw.
#![cfg(feature = "mesh_shaders")]

use ash::vk;
use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::RenderContext,
    },
    window::PrimaryWindow,
};

use wgpu::hal::api::Vulkan as VkApi;

use crate::{
    ash::{mesh_pipelines::MeshPipelines, AshContext},
    compute::dual_contour::MeshletBuffers,
};

#[derive(Debug)]
pub struct MeshShaderNode;

impl Node for MeshShaderNode {
    fn run(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let ash = world.resource::<AshContext>();
        let pipes = world.resource::<MeshPipelines>();
        let mesh = world.resource::<MeshletBuffers>();

        // window size -----------------------------------------------------------
        // `try_query_filtered()` works with &World.  `QueryState::iter()` still
        // needs the state itself mutable, so keep `mut q`, but the world stays `&`.
        let (w, h) = world
            .try_query_filtered::<&Window, With<PrimaryWindow>>()          // ← immut.
            .and_then(|mut q| {
                q.iter(world)                                              // ← still &World
                    .next()
                    .map(|win| (win.resolution.physical_width(),
                                win.resolution.physical_height()))
            })
            .unwrap_or((1, 1)); // fallback before the window is available

        /* -------- record Vulkan commands --------------------------- */
        let vk_cmd = unsafe {
            ctx.command_encoder().as_hal_mut::<VkApi, _, _>(|enc_opt| {
                enc_opt.expect("vk encoder").raw_handle()
            })
        };

        // ── grab the underlying `vk::Buffer` handles coming from wgpu-hal ──
        //
        //  wgpu-hal v24 removed the old public `raw_handle()`/`raw()` helpers,
        //  leaving the `raw` field private.  We still need the plain Vulkan
        //  handles for `cmd_bind_vertex_buffers` / `cmd_draw_indirect`, so we
        //  retrieve them with an unsafe cast (first field in the struct).
        //
        //  SAFETY: the layout of `wgpu_hal::vulkan::Buffer` is considered part
        //  of wgpu-hal’s public ABI for back-ends; the first field is always
        //  the `ash::vk::Buffer` handle.
        let vk_vertices: vk::Buffer = unsafe {
            mesh.vertices.as_hal::<VkApi, _, _>(|b| {
                let ptr = b.expect("Vk buffer") as *const _ as *const vk::Buffer;
                *ptr
            })
        };
        let vk_indirect: vk::Buffer = unsafe {
            mesh.indirect.as_hal::<VkApi, _, _>(|b| {
                let ptr = b.expect("Vk buffer") as *const _ as *const vk::Buffer;
                *ptr
            })
        };


        /* record commands --------------------------------------------- */
        unsafe {
            /* make compute writes visible */
            let barrier = vk::MemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_stage_mask(
                    vk::PipelineStageFlags2::MESH_SHADER_EXT | vk::PipelineStageFlags2::DRAW_INDIRECT,
                )
                .src_access_mask(vk::AccessFlags2::SHADER_WRITE)
                .dst_access_mask(
                    vk::AccessFlags2::INDIRECT_COMMAND_READ
                        | vk::AccessFlags2::VERTEX_ATTRIBUTE_READ
                        | vk::AccessFlags2::SHADER_READ,
                );
            ash.device.cmd_pipeline_barrier2(
                vk_cmd,
                &vk::DependencyInfo::default().memory_barriers(std::slice::from_ref(&barrier)),
            );

            /* dynamic viewport/scissor */
            ash.device.cmd_set_viewport(
                vk_cmd,
                0,
                &[vk::Viewport { x: 0.0, y: 0.0, width: w as f32, height: h as f32, min_depth: 0.0, max_depth: 1.0 }],
            );
            ash.device.cmd_set_scissor(
                vk_cmd,
                0,
                &[vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: w, height: h } }],
            );

            ash.device.cmd_bind_pipeline(vk_cmd, vk::PipelineBindPoint::GRAPHICS, pipes.mesh_pipeline);
            ash.device.cmd_bind_vertex_buffers(vk_cmd, 0, &[vk_vertices], &[0]);
            ash.device.cmd_draw_indirect(
                vk_cmd,
                vk_indirect,
                0,
                1,
                std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
            );
        }
        Ok(())
    }
}
