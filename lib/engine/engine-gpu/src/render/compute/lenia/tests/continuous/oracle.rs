use simulation_kernel::core::{
    boundary::Boundary,
    lenia::{
        LeniaParams,
        continuous::{self, Kernel, Shape},
    },
};
#[test]
fn normalized_periodic_field_and_z_coupling() {
    let size = [7, 7, 7];
    let p = LeniaParams {
        mu: 0.3,
        sigma: 0.12,
        dt: 0.1,
    };
    let k = Kernel::default();
    let uniform = continuous::step(&vec![0.3; 343], size, k, p).unwrap();
    assert!(
        uniform.iter().all(|v| (*v - 0.4).abs() < 1e-6),
        "kernel sums to one"
    );
    let mut initial = vec![0.0; 343];
    initial[3 * 49 + 3 * 7 + 3] = 1.0;
    let p = LeniaParams {
        mu: 0.02,
        sigma: 0.01,
        dt: 0.1,
    };
    let next = continuous::step(&initial, size, k, p).unwrap();
    assert!(
        next[4 * 49 + 3 * 7 + 3] > 0.0,
        "influence reaches adjacent Z plane"
    );
    assert_eq!(
        next[4 * 49 + 3 * 7 + 3],
        next[3 * 49 + 4 * 7 + 3],
        "isotropic axes"
    );
    assert!(continuous::step(&[f32::NAN], [1, 1, 1], k, p).is_err());
    assert!(Kernel { radius: 0, ..k }.validate().is_err());
}
