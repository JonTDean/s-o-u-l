//! engine‑gpu / compute / **mesh_path.rs**
//! ======================================
//! Mesh‑shader **fast‑path** for Dual‑Contouring (optional).
//!
//! The node does *nothing* unless the GPU advertises
//! `Features::MESH_SHADER` **and** the Cargo feature `mesh_shaders` is
//! enabled.  This keeps regular builds light‑weight while exposing a
//! gated playground for bleeding‑edge drivers.
//!
//! ----------------------------------------------------
//! © 2025 Obaven Inc. — Apache‑2.0 OR MIT
//! engine-gpu / compute / **mesh_path.rs** – optional mesh-shader fast-path.

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
    plugin::GlobalStateTextures,
};

const WGSL_SRC: &str =
    include_str!("../../../../../assets/shaders/automatoxel/mesh_path.wgsl");

#[derive(Debug)]
pub struct MeshPathNode;

impl Node for MeshPathNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        /* ───────────── EARLY-OUT GUARD ───────────── */
        let Some(device) = world.get_resource::<RenderDevice>() else { return Ok(()); };
        /* ─────────────────────────────────────────── */

        #[cfg(not(feature = "mesh_shaders"))]
        {
            // feature disabled at compile-time ⇒ no-op
            return Ok(());
        }

        if !device.features().contains(Features::MESH_SHADER) {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| warn!("Mesh-shader fast-path disabled (GPU feature missing)"));
            return Ok(());
        }

        let slices = world.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() {
            return Ok(());
        }

        let tex       = world.resource::<GlobalStateTextures>();
        let images    = world.resource::<RenderAssets<GpuImage>>();
        let ping_view = &images.get(&tex.ping).unwrap().texture_view;

        // Pipeline compile / cache
        let mut pipes   = world.resource_mut::<PipelineCache>();
        let mut shaders = world.resource_mut::<Assets<Shader>>();
        let mut cache   = world.resource_mut::<GpuPipelineCache>();

        let pid = cache.get_or_create("mesh_path", WGSL_SRC, &mut pipes, &mut shaders, device);
        let Some(pipe) = pipes.get_compute_pipeline(pid) else { return Ok(()) };

        // Bind-group
        let bind = device.create_bind_group(
            Some("mesh_path.bind0"),
            &pipe.get_bind_group_layout(0),
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(ping_view),
            }],
        );

        // Work-group count (placeholder heuristic)
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
