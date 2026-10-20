//! Version 1 continuous XYZ Lenia: normalized radial convolution and clipped Euler growth.
//! This finite-lattice model follows Chan's Lenia construction (arXiv:1812.05433);
//! its Gaussian/shell fixtures are not claimed to reproduce a published organism.
use crate::core::boundary::Boundary;
use serde::{Deserialize, Serialize};
mod reference;
pub use reference::step;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    #[default]
    Gaussian,
    Shell,
    /// Published two-dimensional polynomial Orbium model.
    Orbium,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kernel {
    pub radius: u32,
    pub shape: Shape,
    pub boundary: Boundary,
}
impl Default for Kernel {
    fn default() -> Self {
        Self {
            radius: 2,
            shape: Shape::Gaussian,
            boundary: Boundary::Periodic,
        }
    }
}
impl Kernel {
    pub fn validate(self) -> Result<Self, &'static str> {
        if self.shape == Shape::Orbium {
            return if self.radius == 13 && self.boundary == Boundary::Periodic {
                Ok(self)
            } else {
                Err("Orbium requires radius 13 and periodic XY boundaries")
            };
        }
        if !(1..=4).contains(&self.radius) {
            return Err("Continuous radius must be 1..4");
        }
        Ok(self)
    }
    /// Unnormalized radial weight. Normalization includes all offsets, including dead edges.
    pub fn weight(self, distance: f64) -> f64 {
        if distance > self.radius as f64 {
            return 0.0;
        }
        let r = distance / self.radius as f64;
        if self.shape == Shape::Orbium {
            return (4.0 * r * (1.0 - r)).powi(4);
        }
        let (mean, width) = match self.shape {
            Shape::Gaussian => (0.0, 0.5),
            Shape::Shell => (0.65, 0.2),
            Shape::Orbium => unreachable!(),
        };
        (-0.5 * ((r - mean) / width).powi(2)).exp()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub kernel: Kernel,
    pub values: Vec<f32>,
}
impl State {
    pub fn validate(&self, count: usize) -> Result<(), &'static str> {
        self.kernel.validate()?;
        if self.values.len() != count
            || self
                .values
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("Invalid continuous density field");
        }
        Ok(())
    }
    /// Display conversion only; evolution and checkpoints retain floating-point state.
    pub fn bytes(&self) -> Vec<u8> {
        self.values
            .iter()
            .map(|v| (v * 255.0).round() as u8)
            .collect()
    }
}

pub mod orbium;
