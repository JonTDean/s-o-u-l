use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};

#[derive(Resource)]
pub(crate) struct LeniaPipelines {
    pub layout: BindGroupLayout,
    pub evolve: ComputePipeline,
    pub pack: ComputePipeline,
    pub continuous: [ComputePipeline; 2],
}

impl LeniaPipelines {
    pub fn new(device: &RenderDevice) -> Self {
        let entries: Vec<_> = (0..4)
            .map(|binding| BindGroupLayoutEntry {
                binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: if binding == 3 {
                        BufferBindingType::Uniform
                    } else {
                        BufferBindingType::Storage {
                            read_only: binding == 0,
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: if binding == 3 {
                        BufferSize::new(48)
                    } else {
                        None
                    },
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout("lenia.layout", &entries);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("lenia.layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = device
            .wgpu_device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("lenia"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../../assets/shaders/lenia_step.wgsl"
                    ))
                    .into(),
                ),
            });
        let make = |name| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(name),
                compilation_options: default(),
                cache: None,
            })
        };
        let continuous_shader =
            device
                .wgpu_device()
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("lenia.continuous"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/../../../assets/shaders/lenia_continuous.wgsl"
                        ))
                        .into(),
                    ),
                });
        let continuous = ["evolve", "pack"].map(|name| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                module: &continuous_shader,
                entry_point: Some(name),
                compilation_options: default(),
                cache: None,
            })
        });
        Self {
            continuous,
            layout,
            evolve: make("evolve"),
            pack: make("pack"),
        }
    }
}

impl FromWorld for LeniaPipelines {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource::<RenderDevice>())
    }
}
