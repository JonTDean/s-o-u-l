//! Render-graph node that records a single `vkCmdDrawIndirect` using the
//! mesh-shader pipeline.  We insert this node only when the extension
//! is supported and the Cargo feature `mesh_shaders` is enabled.
//!
//! ────────────────────────────────────────────────────────────────
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//

use ash::vk;
use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::RenderContext,
    },
};

use crate::ash::AshContext;
use crate::compute::dual_contour::MeshletBuffers;

#[derive(Debug)]
pub struct MeshShaderNode;

impl Node for MeshShaderNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Resources
        let ash           = world.resource::<AshContext>();
        let buffers       = world.resource::<MeshletBuffers>();
        let mesh_pipelines= world.resource::<super::mesh_pipeline::MeshPipelines>();

        // Raw Vulkan command buffer supplied by Bevy
        let cmd = ctx.command_encoder().raw();

        unsafe {
            ash.device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                mesh_pipelines.mesh_pipeline,
            );
            ash.device
                .cmd_bind_vertex_buffers(cmd, 0, &[buffers.vertices], &[0]);
            ash.device.cmd_draw_indirect(
                cmd,
                buffers.indirect,
                0,
                1,
                std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
            );
        }
        Ok(())
    }
}
