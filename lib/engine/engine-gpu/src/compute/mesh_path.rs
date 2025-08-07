//! Mesh-path compute stage – optional fast-path when `VK_EXT_mesh_shader`
//! hardware is available.
//!
//! The node runs **after** `DualContourNode` and turns the raw vertex
//! stream into an indirect-draw argument buffer.  Right now the shader is
//! an identity pass; later we can add compaction or meshlet generation.
//!
//! ## Bindings (group 0)
//! | index | resource                        | access |
//! |-------|---------------------------------|--------|
//! |   0   | input  vertex buffer            | read   |
//! |   1   | output vertex buffer (may be #0)| rw     |
//! |   2   | `DrawArgs` struct (`DrawIndirect`)| rw  |
//! |   3   | atomic vertex counter           | read   |

use std::time::Instant;

use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};
use wgpu::Features;

use crate::{
    compute::dual_contour::{MeshletBuffers, BYTES_PER_VERT},
    graph::ExtractedGpuSlices,
    pipelines::GpuPipelineCache,
};

/// Render-graph node implementing the mesh-shader fast path.
#[derive(Debug)]
pub struct MeshPathNode;

impl Node for MeshPathNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let begin = Instant::now();

        /* 1 ░ GPU resources ready? */
        let (Some(device), Some(queue)) = (
            world.get_resource::<RenderDevice>(),
            world.get_resource::<RenderQueue>(),
        ) else {
            return Ok(());
        };

        /* 2 ░ Early-out when there were no active slices this frame. */
        if world.resource::<ExtractedGpuSlices>().0.is_empty() {
            return Ok(());
        }

        /* 3 ░ Resolve the compiled compute pipeline from Bevy’s cache. */
        let cache   = world.resource::<GpuPipelineCache>();
        let pipes   = world.resource::<PipelineCache>();
        let Some(&pid) = cache.map.get("mesh_path")        else { return Ok(()); };
        let Some(pipe) = pipes.get_compute_pipeline(pid)   else { return Ok(()); };

        /* 4 ░ Build the bind-group that matches `mesh_path.wgsl`. */
        let buffers = world.resource::<MeshletBuffers>();
        let layout  = BindGroupLayout::from(pipe.get_bind_group_layout(0));

        let bind = device.create_bind_group(
            Some("mesh_path.bind0"),
            &layout,
            &[
                // in-buffer (read-only)
                BindGroupEntry { binding: 0, resource: buffers.vertices.as_entire_binding() },
                // out-buffer (read / write) – for now the same buffer
                BindGroupEntry { binding: 1, resource: buffers.vertices.as_entire_binding() },
                // DrawArgs
                BindGroupEntry { binding: 2, resource: buffers.indirect.as_entire_binding() },
                // atomic counter
                BindGroupEntry { binding: 3, resource: buffers.counter.as_entire_binding() },
            ],
        );

        /* 5 ░ Clear DrawArgs so the shader writes fresh numbers. */
        queue.write_buffer(&buffers.indirect, 0, &[0u8; 16]);

        /* 6 ░ Dispatch: one thread per vertex (group size = 256). */
        let max_vertices = (buffers.capacity / BYTES_PER_VERT) as u32;
        let groups = (max_vertices + 255) / 256;

        let mut pass = ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("mesh_path.compute"),
                timestamp_writes: None,
            });
        pass.set_pipeline(pipe);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(groups, 1, 1);

        /* 7 ░ (Optional) GPU timing – collect if you need it. */
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
