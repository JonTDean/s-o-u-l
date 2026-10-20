//! Independent f64 CPU oracle, intentionally direct convolution instead of GPU indexing code.
use super::Kernel;
use crate::core::{boundary::Boundary, lenia::LeniaParams};
pub fn step(
    values: &[f32],
    size: [u32; 3],
    kernel: Kernel,
    p: LeniaParams,
) -> Result<Vec<f32>, &'static str> {
    kernel.validate()?;
    p.validate()?;
    let count = size
        .iter()
        .try_fold(1usize, |n, v| n.checked_mul(*v as usize));
    if size.contains(&0)
        || count != Some(values.len())
        || values
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err("Invalid continuous field dimensions or values");
    }
    let radius = kernel.radius as i32;
    let mut offsets = Vec::new();
    for z in -radius..=radius {
        for y in -radius..=radius {
            for x in -radius..=radius {
                let weight = kernel.weight(((x * x + y * y + z * z) as f64).sqrt());
                if weight > 0.0 {
                    offsets.push(([x, y, z], weight));
                }
            }
        }
    }
    let norm: f64 = offsets.iter().map(|(_, w)| w).sum();
    let mut next = Vec::with_capacity(values.len());
    for z in 0..size[2] {
        for y in 0..size[1] {
            for x in 0..size[0] {
                let mut potential = 0.0;
                for (offset, weight) in &offsets {
                    let mut pos = [
                        x as i32 + offset[0],
                        y as i32 + offset[1],
                        z as i32 + offset[2],
                    ];
                    if kernel.boundary == Boundary::Periodic {
                        for axis in 0..3 {
                            pos[axis] = pos[axis].rem_euclid(size[axis] as i32);
                        }
                    }
                    if (0..3).all(|a| pos[a] >= 0 && pos[a] < size[a] as i32) {
                        let i = (pos[2] as u32 * size[1] * size[0]
                            + pos[1] as u32 * size[0]
                            + pos[0] as u32) as usize;
                        potential += values[i] as f64 * weight / norm;
                    }
                }
                let growth =
                    2.0 * (-0.5 * ((potential - p.mu as f64) / p.sigma as f64).powi(2)).exp() - 1.0;
                let i = next.len();
                next.push((values[i] as f64 + p.dt as f64 * growth).clamp(0.0, 1.0) as f32);
            }
        }
    }
    Ok(next)
}
