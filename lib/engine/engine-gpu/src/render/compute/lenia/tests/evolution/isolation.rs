use super::super::super::LeniaBuffers;
use super::super::{atlas::Atlas, device::Gpu};
use bevy::render::render_resource::*;
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed, seed, step};

#[test]
fn gpu_lenia_slices_keep_independent_parameters_state_and_atlas_regions() {
    let gpu = Gpu::new();
    let size = [7, 5, 2];
    let mut expected_a = seed(size, LeniaSeed::Blob);
    let mut expected_b = seed(size, LeniaSeed::Ring);
    let mut a = LeniaBuffers::new(&gpu.device, &gpu.pipelines, size, &expected_a).unwrap();
    let mut b = LeniaBuffers::new(&gpu.device, &gpu.pipelines, size, &expected_b).unwrap();
    let atlas = Atlas::new(&gpu, [21, 9, 4]);
    let changed = LeniaParams {
        mu: 0.17,
        sigma: 0.12,
        dt: 0.03,
    };
    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    a.advance(
        &mut encoder,
        &gpu.queue,
        &gpu.pipelines,
        5,
        LeniaParams::default(),
    )
    .unwrap();
    b.advance(&mut encoder, &gpu.queue, &gpu.pipelines, 2, changed)
        .unwrap();
    a.present(
        &mut encoder,
        &gpu.pipelines,
        &atlas.texture,
        Origin3d { x: 2, y: 2, z: 1 },
    );
    b.present(
        &mut encoder,
        &gpu.pipelines,
        &atlas.texture,
        Origin3d { x: 12, y: 2, z: 1 },
    );
    let readback = atlas.read(&gpu, &mut encoder);
    gpu.queue.submit([encoder.finish()]);
    for _ in 0..5 {
        expected_a = step(&expected_a, size, LeniaParams::default()).unwrap();
    }
    for _ in 0..2 {
        expected_b = step(&expected_b, size, changed).unwrap();
    }
    let cells = atlas.cells(&gpu, &readback);
    for z in 0..4usize {
        for y in 0..9usize {
            for x in 0..21usize {
                let value = if (1..3).contains(&z) && (2..7).contains(&y) {
                    let index = ((z - 1) * 5 + y - 2) * 7;
                    if (2..9).contains(&x) {
                        expected_a[index + x - 2]
                    } else if (12..19).contains(&x) {
                        expected_b[index + x - 12]
                    } else {
                        77
                    }
                } else {
                    77
                };
                assert_eq!(cells[(z * 9 + y) * 21 + x], value, "atlas ({x},{y},{z})");
            }
        }
    }
}
