//! Defines convolution kernels for Lenia-like continuous cellular automata.

use bevy::math::IVec2;

/// A convolution kernel with discrete offsets and corresponding weights.
pub struct Kernel {
    pub offsets: Vec<IVec2>,
    pub weights: Vec<f32>,
}

impl Kernel {
    /// Constructs a circular Gaussian kernel with given radius and standard deviation.
    pub fn new_gaussian(radius: i32, sigma: f32) -> Kernel {
        let mut offsets = Vec::new();
        let mut weights = Vec::new();
        let mut sum = 0.0;
        let r_sq = (radius * radius) as f32;
        // include all offsets where x^2 + y^2 <= r^2
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let d2 = (dx*dx + dy*dy) as f32;
                if d2 <= r_sq {
                    let weight = (-d2 / (2.0 * sigma * sigma)).exp();
                    offsets.push(IVec2::new(dx, dy));
                    weights.push(weight);
                    sum += weight;
                }
            }
        }
        // normalize weights so that sum = 1
        if sum > 0.0 {
            for w in weights.iter_mut() {
                *w /= sum;
            }
        }
        Kernel { offsets, weights }
    }
}
