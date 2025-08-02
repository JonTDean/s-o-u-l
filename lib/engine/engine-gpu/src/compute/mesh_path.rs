//! engine-gpu / compute / **mesh_path.rs**
//! ======================================
//! Mesh-shader **fast-path** for Dual-Contouring (optional).
//!
//! Does *nothing* unless the GPU advertises `Features::MESH_SHADER`
//! **and** the Cargo feature `mesh_shaders` is enabled.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

#![allow(clippy::too_many_lines)]

use bevy::{
    log::warn,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::{RenderContext, RenderDevice},
        render_resource::*,
        texture::GpuImage,
    },
};

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
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        /* ── Early-out guards ───────────────────────── */
        let Some(device) = world.get_resource::<RenderDevice>() else { return Ok(()); };

        #[cfg(not(feature = "mesh_shaders"))]
        { return Ok(()); }

        if !device.features().contains(Features::MESH_SHADER) {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| warn!("Mesh-shader path disabled – GPU feature missing"));
            return Ok(());
        }
        /* ──────────────────────────────────────────── */

        let slices = world.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() { return Ok(()); }

        let tex       = world.resource::<GlobalVoxelAtlas>();
        let images    = world.resource::<RenderAssets<GpuImage>>();
        let atlas_view= &images.get(&tex.atlas).unwrap().texture_view;

        let cache = world.resource::<GpuPipelineCache>();
        let pipes = world.resource::<PipelineCache>();
        let Some(&pid) = cache.map.get("mesh_path")      else { return Ok(()); };
        let Some(pipe) = pipes.get_compute_pipeline(pid) else { return Ok(()); };

        // Bind-group
        let bind = device.create_bind_group(
            Some("mesh_path.bind0"),
            &pipe.get_bind_group_layout(0),
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(atlas_view),
            }],
        );

        // Groups: placeholder heuristic (64 voxels / group)
        let vox: u32 = slices.0.iter().map(|s| s.0.size.x * s.0.size.y).sum();
        let groups   = (vox + 63) / 64;

        // Dispatch
        let mut pass = ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("mesh_path.compute"),
                timestamp_writes: None,
            });
        pass.set_pipeline(pipe);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(groups, 1, 1);
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
