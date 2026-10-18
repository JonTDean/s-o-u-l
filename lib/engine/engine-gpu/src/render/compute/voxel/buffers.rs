//! Bounded vertex and indirect buffers shared by surface extraction and drawing.
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};
use bytemuck::{Pod, Zeroable};

pub(crate) const BYTES_PER_VERT: u64 = 32;
pub(crate) const MAX_VERTICES: u32 = 262_140; // Whole triangles, below baseline storage limits.

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct Vertex {
    pub pos: [f32; 4],
    pub nrm: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub(crate) struct SurfaceParams {
    pub origin: [u32; 4],
    pub size: [u32; 4],
    pub world: [f32; 4],
}

#[derive(Resource)]
pub(crate) struct MeshletBuffers {
    pub vertices: Buffer,
    pub counter: Buffer,
    pub indirect: Buffer,
}

impl MeshletBuffers {
    pub(crate) fn new(device: &RenderDevice, vertex_capacity: u32) -> Self {
        let capacity = u64::from(vertex_capacity) * BYTES_PER_VERT;
        let make = |label, size, usage| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        Self {
            vertices: make(
                "voxels.vertices",
                capacity,
                BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_SRC,
            ),
            counter: make(
                "voxels.counter",
                4,
                BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            ),
            indirect: make(
                "voxels.indirect",
                16,
                BufferUsages::STORAGE
                    | BufferUsages::INDIRECT
                    | BufferUsages::COPY_DST
                    | BufferUsages::COPY_SRC,
            ),
        }
    }
}

impl FromWorld for MeshletBuffers {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource::<RenderDevice>(), MAX_VERTICES)
    }
}
