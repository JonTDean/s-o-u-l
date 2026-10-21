//! Pinned published Orbium bicaudatus, with the upstream polynomial model.
use super::{Kernel, Shape};
use crate::core::{boundary::Boundary, lenia::LeniaParams};
pub const KERNEL: Kernel = Kernel {
    radius: 13,
    shape: Shape::Orbium,
    boundary: Boundary::Periodic,
};
pub const PARAMS: LeniaParams = LeniaParams {
    mu: 0.15,
    sigma: 0.014,
    dt: 0.1,
};
/// Place the unmodified upstream density fixture at the middle XY plane.
pub fn seed(size: [u32; 3]) -> Vec<f32> {
    let [w, h, d] = size;
    let mut values = vec![0.0; (w * h * d) as usize];
    if w < 20 || h < 20 || d == 0 {
        return values;
    }
    let cells: Vec<Vec<f32>> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/specimens/orbium.json"
    )))
    .expect("pinned Orbium fixture");
    for (y, row) in cells.iter().enumerate() {
        for (x, value) in row.iter().enumerate() {
            let i = ((d / 2 * h + (h - 20) / 2 + y as u32) * w + (w - 20) / 2 + x as u32) as usize;
            values[i] = *value;
        }
    }
    values
}
