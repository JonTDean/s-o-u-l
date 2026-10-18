//! Explicit bind layouts for portable voxel surface extraction.
use super::{MeshletBuffers, SurfaceParams};
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};

pub(crate) const SURFACE_SHADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/voxel_surface.wgsl"
));

#[derive(Resource)]
pub(crate) struct SurfacePipelines {
    pub layout: BindGroupLayout,
    pub surface: ComputePipeline,
    pub finalize: ComputePipeline,
}

impl SurfacePipelines {
    pub(crate) fn new(device: &RenderDevice) -> Self {
        let entry = |binding, ty| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let storage = BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let layout = device.create_bind_group_layout(
            "voxel_surface.layout",
            &[
                entry(
                    0,
                    BindingType::Texture {
                        sample_type: TextureSampleType::Uint,
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                ),
                entry(1, storage),
                entry(2, storage),
                entry(
                    3,
                    BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(48),
                    },
                ),
                entry(4, storage),
            ],
        );
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("voxel_surface.pipeline_layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = device
            .wgpu_device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("voxel_surface"),
                source: wgpu::ShaderSource::Wgsl(SURFACE_SHADER.into()),
            });
        let make = |entry_point| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: default(),
                cache: None,
            })
        };
        Self {
            layout,
            surface: make("surface"),
            finalize: make("finalize"),
        }
    }

    pub(crate) fn bind(
        &self,
        device: &RenderDevice,
        texture: &TextureView,
        mesh: &MeshletBuffers,
        params: &SurfaceParams,
    ) -> BindGroup {
        let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxel_surface.params"),
            contents: bytemuck::bytes_of(params),
            usage: BufferUsages::UNIFORM,
        });
        device.create_bind_group(
            "voxel_surface.bind",
            &self.layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: mesh.vertices.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: mesh.counter.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: mesh.indirect.as_entire_binding(),
                },
            ],
        )
    }
}

impl FromWorld for SurfacePipelines {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource::<RenderDevice>())
    }
}
