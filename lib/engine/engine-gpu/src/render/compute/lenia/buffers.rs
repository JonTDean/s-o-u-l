use super::LeniaPipelines;
use bevy::render::{render_resource::*, renderer::RenderDevice};
use simulation_kernel::core::lenia::LeniaParams;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuParams {
    pub size: [u32; 4],
    pub growth: [f32; 4],
    pub kernel: [u32; 4],
}

pub(crate) struct LeniaBuffers {
    pub states: [Buffer; 2],
    pub packed: Buffer,
    pub params: Buffer,
    pub binds: [BindGroup; 2],
    pub size: [u32; 3],
    pub stride: u32,
    pub kernel: Option<simulation_kernel::core::lenia::continuous::Kernel>,
    pub current: usize,
    pub tick: u64,
}

impl GpuParams {
    pub fn new(size: [u32; 3], stride: u32, p: LeniaParams) -> Self {
        Self {
            size: [size[0], size[1], size[2], stride],
            growth: [p.mu, p.sigma, p.dt, 0.0],
            kernel: [0; 4],
        }
    }
}

impl LeniaBuffers {
    pub fn new(
        device: &RenderDevice,
        pipelines: &LeniaPipelines,
        size: [u32; 3],
        cells: &[u8],
    ) -> Result<Self, &'static str> {
        let words: Vec<_> = cells.iter().map(|v| u32::from(*v)).collect();
        Self::from_words(device, pipelines, size, &words, None)
    }
    pub fn from_words(
        device: &RenderDevice,
        pipelines: &LeniaPipelines,
        size: [u32; 3],
        cells: &[u32],
        kernel: Option<simulation_kernel::core::lenia::continuous::Kernel>,
    ) -> Result<Self, &'static str> {
        if let Some(k) = kernel {
            k.validate()?;
        }
        let count = size.iter().try_fold(1u64, |n, &v| n.checked_mul(v as u64));
        if size.contains(&0) || count != Some(cells.len() as u64) {
            return Err("invalid Lenia dimensions");
        }
        let stride = size[0].div_ceil(256) * 256;
        let packed_size = stride as u64 * size[1] as u64 * size[2] as u64;
        let limit = device.limits().max_storage_buffer_binding_size as u64;
        if cells.len() as u64 * 4 > limit || packed_size > limit {
            return Err("Lenia exceeds device buffer limit");
        }
        let make = || {
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("lenia.state"),
                contents: bytemuck::cast_slice(cells),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            })
        };
        let states = [make(), make()];
        let packed = device.create_buffer(&BufferDescriptor {
            label: Some("lenia.packed"),
            size: packed_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let params = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("lenia.params"),
            contents: bytemuck::bytes_of(&GpuParams::new(size, stride, LeniaParams::default())),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind = |index: usize| {
            let resources = [&states[index], &states[1 - index], &packed, &params];
            let entries: Vec<_> = resources
                .iter()
                .enumerate()
                .map(|(i, buffer)| BindGroupEntry {
                    binding: i as u32,
                    resource: buffer.as_entire_binding(),
                })
                .collect();
            device.create_bind_group("lenia.bind", &pipelines.layout, &entries)
        };
        let binds = [bind(0), bind(1)];
        Ok(Self {
            states,
            packed,
            params,
            binds,
            size,
            stride,
            kernel,
            current: 0,
            tick: 0,
        })
    }
}
