use super::super::super::LeniaBuffers;
use super::super::device::Gpu;
use bevy::render::render_resource::*;
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed, seed, step};

#[test]
fn gpu_lenia_parameter_changes_apply_to_subsequent_ticks_without_reseeding() {
    let gpu = Gpu::new();
    let size = [17, 13, 3];
    let mut expected = seed(size, LeniaSeed::Blob);
    let mut state = LeniaBuffers::new(&gpu.device, &gpu.pipelines, size, &expected).unwrap();
    for (ticks, params) in [
        (1, LeniaParams::default()),
        (
            2,
            LeniaParams {
                mu: 0.27,
                sigma: 0.08,
                dt: 0.02,
            },
        ),
    ] {
        let mut encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        state
            .advance(&mut encoder, &gpu.queue, &gpu.pipelines, ticks, params)
            .unwrap();
        gpu.queue.submit([encoder.finish()]);
        for _ in 0..ticks {
            expected = step(&expected, size, params).unwrap();
        }
        assert_eq!(gpu.values(&state), expected);
    }
    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    assert!(
        state
            .advance(
                &mut encoder,
                &gpu.queue,
                &gpu.pipelines,
                1,
                LeniaParams {
                    sigma: 0.0,
                    ..LeniaParams::default()
                }
            )
            .is_err()
    );
    gpu.queue.submit([encoder.finish()]);
    assert_eq!(state.tick, 3);
    assert_eq!(gpu.values(&state), expected);
}
