//! View-compatible raster pipelines for the generated voxel vertex stream.
use crate::compute::voxel::BYTES_PER_VERT;
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice, view::ViewUniform},
};
use std::collections::HashMap;

/// Shader assets belong to the application world; only their handle is mirrored.
#[derive(Resource)]
pub(in crate::plugin) struct VoxelShader(Handle<Shader>);

impl VoxelShader {
    pub(in crate::plugin) fn register(world: &mut World) -> Self {
        Self(
            world
                .resource_mut::<Assets<Shader>>()
                .add(Shader::from_wgsl(
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/assets/voxel_mesh.wgsl"
                    )),
                    "voxel_mesh",
                )),
        )
    }
}

#[derive(Resource)]
pub(crate) struct VoxelPipeline {
    pub ids: HashMap<TextureFormat, CachedRenderPipelineId>,
    pub layout: BindGroupLayout,
}

impl FromWorld for VoxelPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let layout = device.create_bind_group_layout(
            "voxel.view",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<
                            super::palette::PaletteUniform,
                        >() as u64),
                    },
                    count: None,
                },
            ],
        );
        let shader = world.resource::<VoxelShader>().0.clone();
        let cache = world.resource::<PipelineCache>();
        let mut ids = HashMap::new();
        for format in [
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba16Float,
        ] {
            let descriptor = RenderPipelineDescriptor {
                label: Some("voxel.surface_draw".into()),
                layout: vec![layout.clone()],
                vertex: VertexState {
                    shader: shader.clone(),
                    entry_point: "vertex".into(),
                    shader_defs: vec![],
                    buffers: vec![VertexBufferLayout {
                        array_stride: BYTES_PER_VERT,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 1,
                            },
                        ],
                    }],
                },
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    entry_point: "fragment".into(),
                    shader_defs: vec![],
                    targets: vec![Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    cull_mode: Some(Face::Back),
                    ..default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::GreaterEqual,
                    stencil: default(),
                    bias: default(),
                }),
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            };
            ids.insert(format, cache.queue_render_pipeline(descriptor));
        }
        Self { ids, layout }
    }
}
