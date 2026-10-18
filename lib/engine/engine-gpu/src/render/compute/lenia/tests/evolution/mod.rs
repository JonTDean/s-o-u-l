use super::super::LeniaBuffers;
use super::device::Gpu;
use bevy::render::render_resource::*;
use simulation_kernel::core::lenia::{LeniaParams, step};
mod isolation;
mod packing;

#[test]
fn gpu_lenia_matches_cpu_for_odd_and_even_tick_batches() {
    let gpu = Gpu::new();
    let size = [13, 9, 3];
    let initial: Vec<u8> = (0..351).map(|i| ((i * 73 + 19) % 256) as u8).collect();
    let mut state = LeniaBuffers::new(&gpu.device, &gpu.pipelines, size, &initial).unwrap();
    let mut expected = initial;
    for steps in [0, 1, 2, 5, 3] {
        let mut encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        state
            .advance(
                &mut encoder,
                &gpu.queue,
                &gpu.pipelines,
                steps,
                LeniaParams::default(),
            )
            .unwrap();
        let readback = gpu.read(&mut encoder, &state.states[state.current]);
        gpu.queue.submit([encoder.finish()]);
        for _ in 0..steps {
            expected = step(&expected, size, LeniaParams::default()).unwrap();
        }
        let actual = gpu.bytes(&readback);
        let values: Vec<u8> = actual
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()) as u8)
            .collect();
        assert_eq!(values, expected, "GPU tick {}", state.tick);
    }
}
