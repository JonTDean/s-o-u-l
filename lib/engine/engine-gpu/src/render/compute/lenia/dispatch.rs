use super::{GpuParams, LeniaBuffers, LeniaPipelines};
use bevy::render::render_resource::*;
use simulation_kernel::core::lenia::LeniaParams;

impl LeniaBuffers {
    /// Each dispatch reads a complete previous tick and swaps before the next.
    pub fn advance(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        pipelines: &LeniaPipelines,
        steps: u32,
        params: LeniaParams,
    ) -> Result<(), &'static str> {
        let params = params.validate()?;
        let mut gpu = GpuParams::new(self.size, self.stride, params);
        if let Some(k) = self.kernel {
            gpu.kernel = [
                k.radius,
                u32::from(k.shape == simulation_kernel::core::lenia::continuous::Shape::Shell),
                u32::from(k.boundary == simulation_kernel::core::boundary::Boundary::Periodic),
                0,
            ];
        }
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&gpu));
        for _ in 0..steps {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("lenia.evolve"),
                timestamp_writes: None,
            });
            pass.set_pipeline(if self.kernel.is_some() {
                &pipelines.continuous[0]
            } else {
                &pipelines.evolve
            });
            pass.set_bind_group(0, &self.binds[self.current], &[]);
            pass.dispatch_workgroups(
                self.size[0].div_ceil(8),
                self.size[1].div_ceil(8),
                self.size[2],
            );
            self.current = 1 - self.current;
            self.tick += 1;
        }
        Ok(())
    }

    /// Pack the current state, then copy only this slice into the render atlas.
    pub fn present(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipelines: &LeniaPipelines,
        texture: &Texture,
        origin: Origin3d,
    ) {
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("lenia.pack"),
                timestamp_writes: None,
            });
            pass.set_pipeline(if self.kernel.is_some() {
                &pipelines.continuous[1]
            } else {
                &pipelines.pack
            });
            pass.set_bind_group(0, &self.binds[self.current], &[]);
            pass.dispatch_workgroups((self.stride / 4).div_ceil(64), self.size[1], self.size[2]);
        }
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.packed,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.stride),
                    rows_per_image: Some(self.size[1]),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin,
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: self.size[0],
                height: self.size[1],
                depth_or_array_layers: self.size[2],
            },
        );
    }
}
