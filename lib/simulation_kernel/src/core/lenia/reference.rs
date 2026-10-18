use super::LeniaParams;

/// Existing finite Gaussian weights, intentionally not renormalized.
pub const KERNEL: [[f32; 3]; 3] = [
    [0.058_549_83, 0.096_532_35, 0.058_549_83],
    [0.096_532_35, 0.159_154_94, 0.096_532_35],
    [0.058_549_83, 0.096_532_35, 0.058_549_83],
];

/// One quantized Euler update; callers validate parameters once per step.
pub fn next_density(current: u8, weighted_density: f32, params: LeniaParams) -> u8 {
    let growth =
        2.0 * (-(weighted_density - params.mu).powi(2) / (2.0 * params.sigma.powi(2))).exp() - 1.0;
    ((current as f32 / 255.0 + params.dt * growth).clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Snapshot update with zero-density boundaries and independent Z planes.
pub fn step(input: &[u8], size: [u32; 3], params: LeniaParams) -> Result<Vec<u8>, &'static str> {
    let params = params.validate()?;
    let [width, height, depth] = size.map(|v| v as usize);
    let count = width.checked_mul(height).and_then(|n| n.checked_mul(depth));
    if count != Some(input.len()) || size.contains(&0) {
        return Err("Lenia input must match nonzero dimensions");
    }
    Ok(input
        .iter()
        .enumerate()
        .map(|(index, &density)| {
            let x = index % width;
            let y = index / width % height;
            let plane = index / (width * height) * width * height;
            let mut u = 0.0;
            for (dy, row) in KERNEL.iter().enumerate() {
                for (dx, weight) in row.iter().enumerate() {
                    let nx = x as isize + dx as isize - 1;
                    let ny = y as isize + dy as isize - 1;
                    if (0..width as isize).contains(&nx) && (0..height as isize).contains(&ny) {
                        u += input[plane + ny as usize * width + nx as usize] as f32 / 255.0
                            * weight;
                    }
                }
            }
            next_density(density, u, params)
        })
        .collect())
}
