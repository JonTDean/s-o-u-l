use super::super::super::LeniaBuffers;
use super::super::{atlas::Atlas, device::Gpu};
use bevy::render::render_resource::*;
use simulation_kernel::core::lenia::{LeniaParams, step};

#[test]
fn gpu_lenia_publishes_odd_width_rows_at_nonzero_atlas_origin() {
    let gpu = Gpu::new();
    let size = [257, 3, 2];
    let initial: Vec<u8> = (0..1542).map(|i| ((i * 71 + 23) % 256) as u8).collect();
    let mut state = LeniaBuffers::new(&gpu.device, &gpu.pipelines, size, &initial).unwrap();
    let atlas = Atlas::new(&gpu, [270, 13, 5]);
    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    state
        .advance(
            &mut encoder,
            &gpu.queue,
            &gpu.pipelines,
            3,
            LeniaParams::default(),
        )
        .unwrap();
    state.present(
        &mut encoder,
        &gpu.pipelines,
        &atlas.texture,
        Origin3d { x: 5, y: 7, z: 2 },
    );
    let readback = atlas.read(&gpu, &mut encoder);
    let packed = gpu.read(&mut encoder, &state.packed);
    gpu.queue.submit([encoder.finish()]);
    let mut expected = initial;
    for _ in 0..3 {
        expected = step(&expected, size, LeniaParams::default()).unwrap();
    }
    let actual = atlas.cells(&gpu, &readback);
    for z in 0..5usize {
        for y in 0..13usize {
            for x in 0..270usize {
                let value = if (2..4).contains(&z) && (7..10).contains(&y) && (5..262).contains(&x)
                {
                    expected[((z - 2) * 3 + y - 7) * 257 + x - 5]
                } else {
                    77
                };
                assert_eq!(actual[(z * 13 + y) * 270 + x], value, "atlas ({x},{y},{z})");
            }
        }
    }
    for (index, row) in gpu.bytes(&packed).chunks_exact(512).enumerate() {
        assert_eq!(&row[..257], &expected[index * 257..(index + 1) * 257]);
        assert!(row[257..].iter().all(|&v| v == 0));
    }
}
