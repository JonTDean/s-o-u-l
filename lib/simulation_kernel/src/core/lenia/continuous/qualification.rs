//! Comparison to the independently retained double-precision FFT trajectory.
use super::{orbium, step};
#[test]
fn published_orbium_matches_fft_reference_through_one_hundred_ticks() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/specimens/orbium-reference.json"
    )))
    .unwrap();
    let mut field = orbium::seed([64, 64, 1]);
    for tick in 0..=100 {
        if let Some(expected) = reference["states"].get(tick.to_string()) {
            let error = field
                .iter()
                .zip(expected.as_array().unwrap())
                .map(|(a, b)| (*a as f64 - b.as_f64().unwrap()).abs())
                .fold(0.0, f64::max);
            eprintln!("Orbium tick {tick}: max FFT reference error {error}");
            assert!(error < 0.0005, "tick {tick}: {error}");
            let mass: f32 = field.iter().sum();
            assert!((20.0..150.0).contains(&mass));
        }
        if tick < 100 {
            field = step(&field, [64, 64, 1], orbium::KERNEL, orbium::PARAMS).unwrap();
        }
    }
}
