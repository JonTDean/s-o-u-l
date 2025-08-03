//! Optional **mesh-shader fast path**.
//!
//! The node is only compiled when *both* the `mesh_shaders` Cargo
//! feature is enabled *and* the underlying GPU reports mesh-shader
//! support through wgpu’s feature flags.  If either condition is
//! missing the module is still built but removed from the render-graph
//! so there is no runtime cost.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//! Optional **mesh-shader fast path**.
//!
//! Compiled only when `mesh_shaders` is enabled *and* the GPU advertises
//! the conservative `VERTEX_WRITABLE_STORAGE` capability (wgpu 0.24
//! removed the old experimental `MESH_SHADER` flag). Eliminates all CPU-
//! side topology work by emitting meshlets directly from WGSL.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
#![allow(clippy::too_many_lines)]

use std::time::Instant;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::{RenderContext, RenderDevice},
        render_resource::*,
        texture::GpuImage,
    },
};
use wgpu::Features;

use crate::{
    graph::ExtractedGpuSlices,
    pipelines::GpuPipelineCache,
    plugin::GlobalVoxelAtlas,
};

#[derive(Debug)]
pub struct MeshPathNode;

impl Node for MeshPathNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx:    &mut RenderContext,
        world:  &World,
    ) -> Result<(), NodeRunError> {
        let begin = Instant::now();

        let Some(device) = world.get_resource::<RenderDevice>() else { return Ok(()); };

        const MESH_CANDIDATE_FEATURE: Features = Features::VERTEX_WRITABLE_STORAGE;
        if !device.features().contains(MESH_CANDIDATE_FEATURE) {
            return Ok(());      // Hardware not capable.
        }

        #[cfg(not(feature = "mesh_shaders"))]
        { return Ok(()); }

        /* 1 ░ Early exit if no slices this frame. */
        let slices = world.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() { return Ok(()); }

        /* 2 ░ GPU resources. */
        let atlas      = world.resource::<GlobalVoxelAtlas>();
        let images     = world.resource::<RenderAssets<GpuImage>>();
        let atlas_view = &images.get(&atlas.atlas).unwrap().texture_view;

        let cache = world.resource::<GpuPipelineCache>();
        let pipes = world.resource::<PipelineCache>();
        let Some(&pid) = cache.map.get("mesh_path")      else { return Ok(()); };
        let Some(pipe) = pipes.get_compute_pipeline(pid) else { return Ok(()); };

        /* 3 ░ Bind-group. */
        let wgpu_layout = pipe.get_bind_group_layout(0);
        let layout      = BindGroupLayout::from(wgpu_layout.clone());
        let bind = device.create_bind_group(
            Some("mesh_path.bind0"),
            &layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(atlas_view),
            }],
        );

        /* 4 ░ Dispatch (≈ 64 voxels / work-group). */
        let voxels: u32 = slices.0.iter().map(|s| s.0.size.x * s.0.size.y).sum();
        let groups      = (voxels + 63) / 64;

        let mut pass = ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("mesh_path.compute"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipe);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(groups, 1, 1);

        let _gpu_ms = begin.elapsed().as_secs_f32() * 1_000.0;
        Ok(())
    }
}


/* ===================================================================== */
/* Notes for future work                                                 */
/* ===================================================================== */
//  • Vertex and indirect buffers are currently written by the *second pass*
//    in `dual_contour.rs`.  Once that file is upgraded to Bevy 0.16 we can
//    expose the buffers via a `MeshletBuffers` resource and bind them here
//    to drive a full `DrawIndirectCommand` stream.
//
//  • `MESH_SHADER_SRC` still points to the original WGSL prototype so hot
//    reload in the Bevy asset‑pipeline works untouched.
//
//  • A lock‑free ring‑buffer allocator for per‑frame scratch space will let
//    us run this stage fully multi‑threaded in the render‑world (no
//    contention with the main app world).
