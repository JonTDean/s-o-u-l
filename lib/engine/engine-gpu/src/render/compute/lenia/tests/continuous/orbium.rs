use super::*;
use simulation_kernel::core::lenia::continuous::orbium;
#[test]
fn published_orbium_gpu_matches_pinned_fft_trajectory() {
    let gpu = Gpu::new();
    let size = [64, 64, 1];
    let initial = orbium::seed(size);
    let words: Vec<_> = initial.iter().map(|v| v.to_bits()).collect();
    let mut state = LeniaBuffers::from_words(
        &gpu.device,
        &gpu.pipelines,
        size,
        &words,
        Some(orbium::KERNEL),
    )
    .unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/specimens/orbium-reference.json"
    )))
    .unwrap();
    for batch in [1, 9, 90] {
        let mut encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        state
            .advance(
                &mut encoder,
                &gpu.queue,
                &gpu.pipelines,
                batch,
                orbium::PARAMS,
            )
            .unwrap();
        let readback = gpu.read(&mut encoder, &state.states[state.current]);
        gpu.queue.submit([encoder.finish()]);
        let bytes = gpu.bytes(&readback);
        let expected = fixture["states"][state.tick.to_string()]
            .as_array()
            .unwrap();
        let error = bytes
            .chunks_exact(4)
            .zip(expected)
            .map(|(a, b)| {
                (f32::from_le_bytes(a.try_into().unwrap()) as f64 - b.as_f64().unwrap()).abs()
            })
            .fold(0.0, f64::max);
        eprintln!("Orbium GPU tick {} max FFT error {error}", state.tick);
        assert!(error < 0.002, "tick {}: {error}", state.tick);
    }
}
