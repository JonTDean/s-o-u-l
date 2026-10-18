//! Exposed-face voxel meshing: atlas regions produce bounded, indirect triangle draws.
use crate::{graph::ExtractedGpuSlices, plugin::GlobalVoxelAtlas};
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
    },
};
mod buffers;
mod pipelines;
pub(crate) use buffers::{BYTES_PER_VERT, MeshletBuffers, SurfaceParams};
pub(crate) use pipelines::SurfacePipelines;

#[derive(Default)]
pub(crate) struct VoxelSurfaceNode;

impl Node for VoxelSurfaceNode {
    fn run(
        &self,
        _: &mut RenderGraphContext,
        context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let device = world.resource::<RenderDevice>();
        let mesh = world.resource::<MeshletBuffers>();
        let pipelines = world.resource::<SurfacePipelines>();
        let images = world.resource::<RenderAssets<GpuImage>>();
        let atlas = world.resource::<GlobalVoxelAtlas>();
        let Some(image) = images.get(&atlas.atlas) else {
            return Ok(());
        };
        let encoder = context.command_encoder();
        encoder.clear_buffer(&mesh.counter, 0, None);
        encoder.clear_buffer(&mesh.indirect, 0, None);
        let mut last_bind = None;
        for slice in &world.resource::<ExtractedGpuSlices>().0 {
            let s = &slice.0;
            let params = SurfaceParams {
                origin: [s.offset.x, s.offset.y, s.layer, 0],
                size: [s.size.x, s.size.y, s.depth.max(1), 0],
                world: [s.offset.x as f32 * 4.0, s.offset.y as f32 * 4.0, 0.0, 4.0],
            };
            let bind = pipelines.bind(device, &image.texture_view, mesh, &params);
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("voxel.surface"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.surface);
                pass.set_bind_group(0, &bind, &[]);
                pass.dispatch_workgroups(
                    s.size.x.div_ceil(4),
                    s.size.y.div_ceil(4),
                    s.depth.max(1).div_ceil(4),
                );
            }
            last_bind = Some(bind);
        }
        if let Some(bind) = last_bind {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("voxel.finalize"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipelines.finalize);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
