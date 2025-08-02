//! Dual-Contouring GPU pass (guaranteed fallback path).
//!
//! Owns **all** CPU-side resources (buffers, bind-groups, pipelines)
//! required by the fallback Dual-Contouring path that runs on *every*
//! GPU.  Buffers live in the render-world; access is safe once
//! `RenderDevice` exists.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssets,
    render_graph::{Node, NodeRunError, RenderGraphContext},
    renderer::{RenderContext, RenderDevice, RenderQueue},
    render_resource::*,
    texture::GpuImage,
};
use bytemuck::{Pod, Zeroable};

use crate::{
    graph::ExtractedGpuSlices,
    pipelines::GpuPipelineCache,
    plugin::GlobalVoxelAtlas,
};

/* ───────────── Vertex format ───────────── */
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub pos: Vec3,   // world-space position
    pub nrm: Vec3,   // world-space normal
    pub mat: u32,    // future material id
}

/* ───────────── Constants ───────────── */
pub const WG_SIZE: (u32, u32, u32) = (8, 8, 8);
pub const MAX_QUADS:  u32 = 5;                       // conservative per-voxel
pub const BYTES_PER_VERT: u64 = std::mem::size_of::<Vertex>() as u64;
pub const MAX_VOXELS: u32 = 1024 * 1024;

/* ───────────── Per-frame GPU scratch buffers ───────────── */
#[derive(Resource)]
pub struct MeshletBuffers {
    pub vertices: Buffer,
    pub counter:  Buffer,
    pub indirect: Buffer,
    #[allow(dead_code)] pub capacity: u64,
}

impl MeshletBuffers {
    pub fn new(device: &RenderDevice, voxel_cap: u32) -> Self {
        let max_vertices = voxel_cap as u64 * MAX_QUADS as u64 * 6;
        let size_bytes   = max_vertices * BYTES_PER_VERT;

        let vertices = device.create_buffer(&BufferDescriptor {
            label: Some("dc.vertices"),
            size: size_bytes,
            usage: BufferUsages::STORAGE
                | BufferUsages::VERTEX
                | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let counter = device.create_buffer(&BufferDescriptor {
            label: Some("dc.counter"),
            size: 4,
            usage: BufferUsages::STORAGE
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let indirect = device.create_buffer(&BufferDescriptor {
            label: Some("dc.indirect"),
            size: std::mem::size_of::<DrawIndirect>() as u64,
            usage: BufferUsages::INDIRECT
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self { vertices, counter, indirect, capacity: size_bytes }
    }
}

/* ───────────── Draw-indirect struct ───────────── */
#[repr(C)]
#[derive(Clone, Copy)]
struct DrawIndirect {
    vertex_count:  u32,
    instance_count: u32,
    first_vertex:  u32,
    first_instance: u32,
}

/* ───────────── Render-graph node ───────────── */
#[derive(Debug)]
pub struct DualContourNode;

impl Node for DualContourNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // 0 ░ Resources … bail early if not yet ready
        let (Some(device), Some(queue)) = (
            world.get_resource::<RenderDevice>(),
            world.get_resource::<RenderQueue>(),
        ) else { return Ok(()); };

        let slices = world.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() { return Ok(()); }

        let images = world.resource::<RenderAssets<GpuImage>>();
        let tex    = world.resource::<GlobalVoxelAtlas>();
        let cache  = world.resource::<GpuPipelineCache>();
        let pipes  = world.resource::<PipelineCache>();
        let mesh   = world.resource::<MeshletBuffers>();

        // 1 ░ Pipeline ready?
        let Some(&pid) = cache.map.get("dual_contour") else { return Ok(()); };
        let Some(pipe) = pipes.get_compute_pipeline(pid) else { return Ok(()); };

        // 2 ░ Bind-group
        let layout  = BindGroupLayout::from(pipe.get_bind_group_layout(0));
        let entries = [
            // storage-3D atlas ─ binding 0
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(
                    &images.get(&tex.atlas).unwrap().texture_view),
            },
            // append vertex buffer ─ binding 1
            BindGroupEntry { binding: 1, resource: mesh.vertices.as_entire_binding() },
            // atomic counter ─ binding 2
            BindGroupEntry { binding: 2, resource: mesh.counter.as_entire_binding() },
        ];
        let bind = device.create_bind_group(Some("dc.bind0"), &layout, &entries);

        // 3 ░ Reset atomics + indirect
        queue.write_buffer(&mesh.counter, 0, bytemuck::bytes_of(&0u32));
        queue.write_buffer(&mesh.indirect, 4, bytemuck::bytes_of(&1u32)); // instance_count
        queue.write_buffer(&mesh.indirect, 8, bytemuck::bytes_of(&0u32));
        queue.write_buffer(&mesh.indirect, 12, bytemuck::bytes_of(&0u32));

        // 4 ░ Work-group extents from largest slice
        let w  = slices.0.iter().map(|s| s.0.size.x).max().unwrap();
        let h  = slices.0.iter().map(|s| s.0.size.y).max().unwrap();
        let gx = (w + WG_SIZE.0 - 1) / WG_SIZE.0;
        let gy = (h + WG_SIZE.1 - 1) / WG_SIZE.1;

        // 5 ░ Dispatch
        {
            let mut pass = ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor {
                label: Some("dual_contour.compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }

        // 6 ░ Copy vertex-count → indirect buffer
        ctx.command_encoder()
            .copy_buffer_to_buffer(&mesh.counter, 0, &mesh.indirect, 0, 4);

        Ok(())
    }
}
