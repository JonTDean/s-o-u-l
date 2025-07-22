//! Definition of an Orbium species in Lenia (one of the lifeforms found by Bert Chan).

use super::super::kernel::Kernel;

/// Parameters for the "Orbium" Lenia creature (a typical self-propagating pattern in Lenia).
pub struct Orbium {
    pub radius: i32,
    pub kernel: Kernel,
    pub mu: f32,
    pub sigma: f32,
    pub time_step: f32,
}

impl Default for Orbium {
    fn default() -> Self {
        // Example parameters: radius 15, mu and sigma define growth curve peak and width, dt time step.
        let radius = 15;
        Orbium {
            radius,
            kernel: Kernel::new_gaussian(radius, 4.0), // Gaussian kernel with sigma_k ~4.0
            mu: 0.15,
            sigma: 0.015,
            time_step: 0.1,
        }
    }
}

impl Orbium {
    /// Growth function G based on the convolution result.
    /// Typically G(x) = 2 * exp(-((x - mu)/sigma)^2) - 1 (produces values in [-1,1]).
    pub fn growth_curve(&self, conv_value: f32) -> f32 {
        let norm = (conv_value - self.mu) / self.sigma;
        2.0 * (-norm * norm).exp() - 1.0
    }

    /// Convenience method to perform one update step on a given world.
    pub fn step_world(&self, world: &mut engine_core::core::world::World2D) {
        super::super::evolver::evolve_lenia(world, self);
    }
}
