use super::device::Gpu;
use bevy::render::render_resource::*;

pub(super) struct Atlas {
    pub texture: Texture,
    pub size: [u32; 3],
    pub stride: u32,
}

impl Atlas {
    pub fn new(gpu: &Gpu, size: [u32; 3]) -> Self {
        let extent = Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: size[2],
        };
        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("lenia.test.atlas"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::R8Uint,
            usage: TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &vec![77; size.iter().product::<u32>() as usize],
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size[0]),
                rows_per_image: Some(size[1]),
            },
            extent,
        );
        Self {
            texture,
            size,
            stride: size[0].div_ceil(256) * 256,
        }
    }

    pub fn read(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder) -> Buffer {
        let buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("lenia.test.texture-readback"),
            size: (self.stride * self.size[1] * self.size[2]) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.stride),
                    rows_per_image: Some(self.size[1]),
                },
            },
            Extent3d {
                width: self.size[0],
                height: self.size[1],
                depth_or_array_layers: self.size[2],
            },
        );
        buffer
    }

    pub fn cells(&self, gpu: &Gpu, buffer: &Buffer) -> Vec<u8> {
        gpu.bytes(buffer)
            .chunks_exact(self.stride as usize)
            .flat_map(|row| row[..self.size[0] as usize].iter().copied())
            .collect()
    }
}
