use serde::{Deserialize, Serialize};

/// Growth parameters for the finite, non-normalized 3×3 XY kernel.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeniaParams {
    pub mu: f32,
    pub sigma: f32,
    pub dt: f32,
}

impl Default for LeniaParams {
    fn default() -> Self {
        Self {
            mu: 0.30,
            sigma: 0.06,
            dt: 0.05,
        }
    }
}

impl LeniaParams {
    /// Reject non-finite or out-of-range values before any state is updated.
    pub fn validate(self) -> Result<Self, &'static str> {
        if !(0.0..=1.0).contains(&self.mu)
            || !(0.001..=1.0).contains(&self.sigma)
            || !(0.0..=1.0).contains(&self.dt)
        {
            return Err("Lenia requires mu 0..1, sigma 0.001..1, dt 0..1");
        }
        Ok(self)
    }
}
