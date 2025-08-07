use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::{ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

use crate::{
    compute::dual_contour::MeshletBuffers,
    plugin::VoxelPipeline,
};

#[derive(Debug)]
pub struct VoxelDrawNode;

impl Node for VoxelDrawNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx:   &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        /* 1 ─ pull GPU resources */
        let (Some(cache), Some(vx), Some(mesh)) = (
            world.get_resource::<PipelineCache>(),
            world.get_resource::<VoxelPipeline>(),
            world.get_resource::<MeshletBuffers>(),
        ) else { return Ok(()); };

        let Some(pipeline) = cache.get_render_pipeline(vx.id) else {
            return Ok(());                         // WGSL not ready
        };

        /* 2 ─ find any entity that has both ViewTarget & ViewUniformOffset */
        let view_entity = if let Some(e) = world
            .iter_entities()                       // read‑only iterator :contentReference[oaicite:0]{index=0}
            .find(|e| e.contains::<ViewTarget>() && e.contains::<ViewUniformOffset>()) {
                e.id()
            } else {
                return Ok(());                     // nothing extracted yet
            };

        let view_target = world.get::<ViewTarget>(view_entity).unwrap();
        let view_offset = world.get::<ViewUniformOffset>(view_entity).unwrap();

        /* 3 ─ build transient bind‑group BEFORE the pass */
        let view_uniforms = world.resource::<ViewUniforms>();
        let device        = world.resource::<RenderDevice>();

        let binding = view_uniforms
            .uniforms
            .binding()                             // DynamicUniformBuffer helper 
            .expect("ViewUniforms not uploaded yet");

        // convert wgpu layout → Bevy wrapper
        let layout = BindGroupLayout::from(pipeline.get_bind_group_layout(0).clone());
        let view_bg = device.create_bind_group(
            None,
            &layout,
            &[BindGroupEntry { binding: 0, resource: binding }],
        );

        /* 4 ─ begin pass in inner scope so borrow ends first */
        {
            let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("voxel_debug_pass"),
                color_attachments: &[Some(view_target.get_color_attachment())],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_render_pipeline(pipeline);
            pass.set_vertex_buffer(0, mesh.vertices.slice(..));
            pass.set_bind_group(0, &view_bg, &[view_offset.offset]);     // ✓ lifetimes
            pass.draw_indirect(&mesh.indirect, 0);
        }

        Ok(())
    }
}
