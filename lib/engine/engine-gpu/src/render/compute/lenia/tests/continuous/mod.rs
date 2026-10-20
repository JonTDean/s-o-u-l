use super::{super::LeniaBuffers, device::Gpu};
use bevy::render::render_resource::*;
use simulation_kernel::core::{
    boundary::Boundary,
    lenia::{
        LeniaParams,
        continuous::{self, Kernel, Shape},
    },
};
#[test]
fn continuous_gpu_matches_independent_oracle_and_restored_continuation() {
    let gpu = Gpu::new();
    let size = [9, 7, 5];
    for shape in [Shape::Gaussian, Shape::Shell] {
        for boundary in [Boundary::Dead, Boundary::Periodic] {
            for radius in [1, 2, 4] {
                let kernel = Kernel {
                    radius,
                    shape,
                    boundary,
                };
                let mut expected: Vec<f32> = (0..315)
                    .map(|i| ((i * 73 + 19) % 997) as f32 / 997.0)
                    .collect();
                let words: Vec<_> = expected.iter().map(|v| v.to_bits()).collect();
                let mut state = LeniaBuffers::from_words(
                    &gpu.device,
                    &gpu.pipelines,
                    size,
                    &words,
                    Some(kernel),
                )
                .unwrap();
                for batch in [1, 2, 3] {
                    let p = LeniaParams {
                        mu: 0.35,
                        sigma: 0.15,
                        dt: 0.02,
                    };
                    let mut encoder = gpu
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor::default());
                    state
                        .advance(&mut encoder, &gpu.queue, &gpu.pipelines, batch, p)
                        .unwrap();
                    let readback = gpu.read(&mut encoder, &state.states[state.current]);
                    gpu.queue.submit([encoder.finish()]);
                    let actual: Vec<_> = gpu
                        .bytes(&readback)
                        .chunks_exact(4)
                        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                        .collect();
                    for _ in 0..batch {
                        expected = continuous::step(&expected, size, kernel, p).unwrap();
                    }
                    let error = actual
                        .iter()
                        .zip(&expected)
                        .map(|(a, b)| (a - b).abs())
                        .fold(0.0, f32::max);
                    let worst = actual
                        .iter()
                        .zip(&expected)
                        .enumerate()
                        .max_by(|(_, (a, b)), (_, (c, d))| {
                            (**a - **b).abs().total_cmp(&(**c - **d).abs())
                        })
                        .unwrap();
                    assert!(
                        error < 0.00001,
                        "{kernel:?} tick {} max error {error} worst {worst:?}",
                        state.tick
                    );
                    assert!(
                        actual.iter().any(|v| (v * 255.0).fract().abs() > 0.01),
                        "not byte quantized"
                    );
                    let words: Vec<_> = actual.iter().map(|v| v.to_bits()).collect();
                    let tick = state.tick;
                    state = LeniaBuffers::from_words(
                        &gpu.device,
                        &gpu.pipelines,
                        size,
                        &words,
                        Some(kernel),
                    )
                    .unwrap();
                    state.tick = tick;
                }
            }
        }
    }
}

mod oracle;

mod orbium;
