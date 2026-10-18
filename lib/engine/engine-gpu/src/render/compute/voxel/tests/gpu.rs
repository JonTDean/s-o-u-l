//! Headless GPU execution using the same buffers, pipelines and bindings as the app.
use super::super::{MeshletBuffers, SurfaceParams, SurfacePipelines};
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
    tasks::block_on,
};

pub(super) fn run(
    cells: &[u8],
    size: [u32; 3],
    capacity: u32,
    origin: [u32; 3],
) -> (Vec<[f32; 8]>, [u32; 4]) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..default()
    });
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("Vulkan adapter required for GPU validation");
    eprintln!("GPU readback adapter: {:?}", adapter.get_info());
    let (raw, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("soul.surface.test"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: default(),
        },
        None,
    ))
    .expect("GPU device");
    raw.set_device_lost_callback(|reason, message| {
        eprintln!("GPU device lost: {reason:?}: {message}")
    });
    let device = RenderDevice::from(raw);
    let pipelines = SurfacePipelines::new(&device);
    let mesh = MeshletBuffers::new(&device, capacity);
    let extent = Extent3d {
        width: size[0] + origin[0] + 1,
        height: size[1] + origin[1] + 1,
        depth_or_array_layers: size[2] + origin[2] + 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("surface.test.atlas"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Uint,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d {
                x: origin[0],
                y: origin[1],
                z: origin[2],
            },
            aspect: TextureAspect::All,
        },
        cells,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size[0]),
            rows_per_image: Some(size[1]),
        },
        Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: size[2],
        },
    );
    let params = SurfaceParams {
        origin: [origin[0], origin[1], origin[2], 0],
        size: [size[0], size[1], size[2], 0],
        world: [10.0, 20.0, 30.0, 4.0],
    };
    let bind = pipelines.bind(
        &device,
        &texture.create_view(&TextureViewDescriptor::default()),
        &mesh,
        &params,
    );
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    encoder.clear_buffer(&mesh.counter, 0, None);
    for (pipeline, groups) in [
        (
            &pipelines.surface,
            [
                size[0].div_ceil(4),
                size[1].div_ceil(4),
                size[2].div_ceil(4),
            ],
        ),
        (&pipelines.finalize, [1, 1, 1]),
    ] {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(groups[0], groups[1], groups[2]);
    }
    let readback = device.create_buffer(&BufferDescriptor {
        label: Some("surface.test.readback"),
        size: mesh.vertices.size() + 16,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&mesh.indirect, 0, &readback, 0, 16);
    encoder.copy_buffer_to_buffer(&mesh.vertices, 0, &readback, 16, mesh.vertices.size());
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(MapMode::Read, move |result| sender.send(result).unwrap());
    device.wgpu_device().poll(wgpu::Maintain::Wait);
    receiver.recv().unwrap().unwrap();
    let data = readback.slice(..).get_mapped_range();
    let draw = bytemuck::pod_read_unaligned::<[u32; 4]>(&data[..16]);
    let vertices = data[16..16 + draw[0] as usize * 32]
        .chunks_exact(32)
        .map(bytemuck::pod_read_unaligned::<[f32; 8]>)
        .collect();
    (vertices, draw)
}
