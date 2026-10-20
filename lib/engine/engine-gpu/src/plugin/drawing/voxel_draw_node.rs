//! Draw voxel geometry into the current 3D view, preserving depth and camera isolation.
use crate::{compute::voxel::MeshletBuffers, plugin::VoxelPipeline};
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        view::{ViewDepthTexture, ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

#[derive(Default)]
pub(crate) struct VoxelDrawNode;

impl ViewNode for VoxelDrawNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewUniformOffset,
        &'static ViewDepthTexture,
    );

    fn run(
        &self,
        _: &mut RenderGraphContext,
        context: &mut RenderContext,
        (target, offset, depth): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let images = world
            .resource::<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>(
            );
        let atlas = world.resource::<crate::plugin::GlobalVoxelAtlas>();
        if images.get(&atlas.atlas).is_none()
            || world
                .resource::<crate::graph::ExtractedGpuSlices>()
                .0
                .is_empty()
        {
            return Ok(());
        }
        let pipeline = world.resource::<VoxelPipeline>();
        let Some(id) = pipeline.ids.get(&target.main_texture_format()) else {
            return Ok(());
        };
        let Some(draw) = world.resource::<PipelineCache>().get_render_pipeline(*id) else {
            return Ok(());
        };
        let uniforms = world.resource::<ViewUniforms>();
        let Some(binding) = uniforms.uniforms.binding() else {
            return Ok(());
        };
        let device = world.resource::<RenderDevice>();
        let palette = world.resource::<super::palette::VoxelPalette>();
        let palette = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxel.palette"),
            contents: bytemuck::bytes_of(&palette.0),
            usage: BufferUsages::UNIFORM,
        });
        let group = device.create_bind_group(
            "voxel.view.bind",
            &pipeline.layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: binding,
                },
                BindGroupEntry {
                    binding: 1,
                    resource: palette.as_entire_binding(),
                },
            ],
        );
        let mesh = world.resource::<MeshletBuffers>();
        let attachments = [Some(target.get_color_attachment())];
        let mut pass = context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("voxel.surface_draw"),
            color_attachments: &attachments,
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_render_pipeline(draw);
        pass.set_vertex_buffer(0, mesh.vertices.slice(..));
        pass.set_bind_group(0, &group, &[offset.offset]);
        pass.draw_indirect(&mesh.indirect, 0);
        world
            .resource::<crate::VoxelRenderStatus>()
            .mark_draw_recorded();
        Ok(())
    }
}
