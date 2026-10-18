use super::super::LeniaPipelines;
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
    tasks::block_on,
};

pub(super) struct Gpu {
    pub device: RenderDevice,
    pub queue: wgpu::Queue,
    pub pipelines: LeniaPipelines,
}

impl Gpu {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..default()
        });
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("Vulkan adapter required");
        eprintln!("Lenia GPU adapter: {:?}", adapter.get_info());
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("lenia.test"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: default(),
            },
            None,
        ))
        .expect("Lenia device");
        let device = RenderDevice::from(device);
        let pipelines = LeniaPipelines::new(&device);
        Self {
            device,
            queue,
            pipelines,
        }
    }

    pub fn read(&self, encoder: &mut wgpu::CommandEncoder, source: &Buffer) -> Buffer {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("lenia.readback"),
            size: source.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(source, 0, &buffer, 0, source.size());
        buffer
    }

    pub fn bytes(&self, buffer: &Buffer) -> Vec<u8> {
        let (tx, rx) = std::sync::mpsc::channel();
        buffer
            .slice(..)
            .map_async(MapMode::Read, move |result| tx.send(result).unwrap());
        self.device.wgpu_device().poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();
        let bytes = buffer.slice(..).get_mapped_range().to_vec();
        buffer.unmap();
        bytes
    }
}
