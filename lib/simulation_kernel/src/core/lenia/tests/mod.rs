use super::*;
mod numerical;

#[test]
fn growth_clamps_quantizes_and_preserves_zero_dt() {
    let p = LeniaParams::default();
    assert_eq!(next_density(128, p.mu, p), 141);
    assert_eq!(next_density(250, p.mu, p), 255);
    assert_eq!(next_density(3, 1.0, p), 0);
    assert_eq!(next_density(128, 0.0, p), 115);
    for density in 0..=255 {
        assert_eq!(
            next_density(density, 0.7, LeniaParams { dt: 0.0, ..p }),
            density
        );
    }
}

#[test]
fn parameters_and_shape_fail_closed() {
    let p = LeniaParams::default();
    for invalid in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        assert!(LeniaParams { mu: invalid, ..p }.validate().is_err());
        assert!(LeniaParams { dt: invalid, ..p }.validate().is_err());
        assert!(
            LeniaParams {
                sigma: invalid,
                ..p
            }
            .validate()
            .is_err()
        );
    }
    assert!(LeniaParams { sigma: 0.0, ..p }.validate().is_err());
    assert!(step(&[], [0, 1, 1], p).is_err());
    assert!(step(&[1], [2, 1, 1], p).is_err());
    assert!(step(&[1], [u32::MAX; 3], p).is_err());
}

#[test]
fn every_xy_neighbor_uses_its_original_offset() {
    let p = LeniaParams {
        mu: 0.07,
        sigma: 0.03,
        dt: 0.1,
    };
    for y in 0..3 {
        for x in 0..3 {
            let mut cells = vec![0; 9];
            cells[4] = 100;
            cells[y * 3 + x] = 255;
            let expected = if x == 1 && y == 1 {
                KERNEL[1][1]
            } else {
                100.0 / 255.0 * KERNEL[1][1] + KERNEL[y][x]
            };
            assert_eq!(
                step(&cells, [3, 3, 1], p).unwrap()[4],
                next_density(cells[4], expected, p),
                "neighbor ({x},{y})"
            );
        }
    }
}

#[test]
fn boundaries_are_zero_and_z_planes_are_independent() {
    let p = LeniaParams::default();
    let cells = [180, 0, 0, 0, 255, 255, 255, 255];
    let output = step(&cells, [2, 2, 2], p).unwrap();
    assert_eq!(
        output[0],
        next_density(180, 180.0 / 255.0 * KERNEL[1][1], p)
    );
    assert_eq!(&output[..4], step(&cells[..4], [2, 2, 1], p).unwrap());
    assert_eq!(&output[4..], step(&cells[4..], [2, 2, 1], p).unwrap());
}
