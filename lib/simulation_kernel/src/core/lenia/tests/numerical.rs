use super::*;

/// Deliberately independent f64 implementation with explicit scalar weights.
fn oracle(input: &[u8], [width, height, depth]: [u32; 3], p: LeniaParams) -> Vec<u8> {
    let mut output = Vec::new();
    for z in 0..depth as i32 {
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let index = ((z * height as i32 + y) * width as i32 + x) as usize;
                let mut potential = 0.0_f64;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let weight = match (dx == 0, dy == 0) {
                            (true, true) => 0.15915494,
                            (false, false) => 0.05854983,
                            _ => 0.09653235,
                        };
                        potential += input[((z * height as i32 + ny) * width as i32 + nx) as usize]
                            as f64
                            / 255.0
                            * weight;
                    }
                }
                let growth = 2.0
                    * (-(potential - p.mu as f64).powi(2) / (2.0 * (p.sigma as f64).powi(2))).exp()
                    - 1.0;
                output.push(
                    ((input[index] as f64 / 255.0 + p.dt as f64 * growth).clamp(0.0, 1.0) * 255.0)
                        .round() as u8,
                );
            }
        }
    }
    output
}

#[test]
fn seeded_multitick_reference_matches_independent_numerics() {
    let size = [17, 13, 3];
    for pattern in [LeniaSeed::Blob, LeniaSeed::Ring] {
        let initial = seed(size, pattern);
        let mut actual = initial.clone();
        let mut expected = initial.clone();
        for _ in 0..20 {
            actual = step(&actual, size, LeniaParams::default()).unwrap();
            expected = oracle(&expected, size, LeniaParams::default());
            assert_eq!(actual, expected);
        }
        assert_ne!(actual, initial);
    }
}

#[test]
fn fixtures_are_safe_for_small_shapes_and_distinct() {
    for size in [[1, 1, 1], [2, 3, 1], [7, 5, 3]] {
        let cells = seed(size, LeniaSeed::Blob);
        assert_eq!(cells.len(), size.iter().product::<u32>() as usize);
        assert!(cells.iter().any(|&v| v != 0));
    }
    assert_ne!(
        seed([32, 32, 8], LeniaSeed::Blob),
        seed([32, 32, 8], LeniaSeed::Ring)
    );
}
