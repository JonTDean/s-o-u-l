//! GPU **Dual-Contouring** fallback path – always available.
//!
//! Converts active voxels into a compact vertex stream that a single
//! indirect draw can consume.  Scratch buffers are reused every frame
//! and sized once during start-up.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

use std::time::Instant;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        render_resource::*,
        texture::GpuImage,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::{
    graph::ExtractedGpuSlices,
    pipelines::GpuPipelineCache,
    plugin::GlobalVoxelAtlas,
};

/* ───────────── Vertex POD ───────────── */
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub nrm: Vec3,
}

/* ───────────── Constants ───────────── */
pub const WG_SIZE: (u32, u32, u32) = (8, 8, 8);
pub const MAX_QUADS:  u32 = 5;
pub const BYTES_PER_VERT: u64 = std::mem::size_of::<Vertex>() as u64;
pub const MAX_VOXELS: u32 = 1024 * 1024;

/* ───────────── Transient buffers ───── */
#[derive(Resource)]
pub struct MeshletBuffers {
    pub vertices: Buffer,
    pub counter:  Buffer,
    pub indirect: Buffer,
    pub capacity: u64,
}

impl MeshletBuffers {
    /// Allocate enough scratch space for `voxel_cap` active voxels.
    pub fn new(device: &RenderDevice, voxel_cap: u32) -> Self {
        let max_vertices = voxel_cap as u64 * MAX_QUADS as u64 * 6;
        let size_bytes   = max_vertices * BYTES_PER_VERT;

        let vertices = device.create_buffer(&BufferDescriptor {
            label: Some("dc.vertices"),
            size:  size_bytes,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let counter = device.create_buffer(&BufferDescriptor {
            label: Some("dc.counter"),
            size: 4,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let indirect = device.create_buffer(&BufferDescriptor {
            label: Some("dc.indirect"),
            size: std::mem::size_of::<DrawIndirect>() as u64,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self { vertices, counter, indirect, capacity: size_bytes }
    }
}

/* ───────────── DrawIndirect POD ────── */
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct DrawIndirect {
    vertex_count:   u32,
    instance_count: u32,
    first_vertex:   u32,
    first_instance: u32,
}

/* ───────────── Render-graph node ───── */
#[derive(Debug)]
pub struct DualContourNode;

impl Node for DualContourNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx:    &mut RenderContext,
        world:  &World,
    ) -> Result<(), NodeRunError> {
        let begin = Instant::now();

        /* 0 ░ Bail out if the render-world isn’t ready yet. */
        let (Some(device), Some(queue)) = (
            world.get_resource::<RenderDevice>(),
            world.get_resource::<RenderQueue>(),
        ) else { return Ok(()); };

        /* 1 ░ Skip frames with no active slices. */
        let slices = world.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() { return Ok(()); }

        /* 2 ░ Fetch GPU resources. */
        let images = world.resource::<RenderAssets<GpuImage>>();
        let atlas  = world.resource::<GlobalVoxelAtlas>();
        let cache  = world.resource::<GpuPipelineCache>();
        let pipes  = world.resource::<PipelineCache>();
        let mesh   = world.resource::<MeshletBuffers>();

        /* 3 ░ Resolve the compute pipeline. */
        let Some(&pid) = cache.map.get("dual_contour") else { return Ok(()); };
        let Some(pipe) = pipes.get_compute_pipeline(pid) else { return Ok(()); };

        /* 4 ░ Create the bind-group for this frame. */
        let wgpu_layout = pipe.get_bind_group_layout(0);
        let layout      = BindGroupLayout::from(wgpu_layout.clone());
        let bind = device.create_bind_group(
            Some("dc.bind0"),
            &layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &images.get(&atlas.atlas).unwrap().texture_view),
                },
                BindGroupEntry { binding: 1, resource: mesh.vertices.as_entire_binding() },
                BindGroupEntry { binding: 2, resource: mesh.counter .as_entire_binding() },
            ],
        );

        /* 5 ░ Reset atomics + indirect header. */
        queue.write_buffer(&mesh.counter, 0, bytemuck::bytes_of(&0u32));
        queue.write_buffer(&mesh.indirect, 4, bytemuck::bytes_of(&1u32)); // instance_count
        queue.write_buffer(&mesh.indirect, 8, bytemuck::bytes_of(&0u32));
        queue.write_buffer(&mesh.indirect, 12,bytemuck::bytes_of(&0u32));

        /* 6 ░ Compute grid = largest slice in this frame. */
        let w  = slices.0.iter().map(|s| s.0.size.x).max().unwrap();
        let h  = slices.0.iter().map(|s| s.0.size.y).max().unwrap();
        let gx = (w + WG_SIZE.0 - 1) / WG_SIZE.0;
        let gy = (h + WG_SIZE.1 - 1) / WG_SIZE.1;

        {
            let mut pass = ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor {
                label: Some("dual_contour.compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }

        /* 7 ░ Copy vertex count → indirect buffer. */
        ctx.command_encoder()
            .copy_buffer_to_buffer(&mesh.counter, 0, &mesh.indirect, 0, 4);

        /* 8 ░ (Optional) timing – collected here but written elsewhere. */
        let _gpu_ms = begin.elapsed().as_secs_f32() * 1_000.0;

        Ok(())
        
    }
}
