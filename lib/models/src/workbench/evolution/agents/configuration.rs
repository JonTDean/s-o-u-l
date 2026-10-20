use super::{Ant, Boid, ChaCha8Rng, Flock, Foraging};
use crate::workbench::{catalog::Kind, experiment::config::Settings};
use glam::Vec2;
use rand::SeedableRng;
impl Flock {
    /// Rebuild the configured population; live parameter edits do not call this.
    pub fn configured(settings: &Settings) -> Self {
        let offset =
            ((settings.random_seed ^ Settings::for_kind(Kind::Boids).random_seed) % 360) as f32;
        let boids = (0..settings.agent_count)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / settings.agent_count as f32
                    + offset.to_radians();
                Boid {
                    position: Vec2::splat(32.0) + Vec2::from_angle(angle) * 18.0,
                    velocity: Vec2::from_angle(angle + 1.0),
                }
            })
            .collect();
        Self { boids }
    }
}
impl Foraging {
    /// Restore home, food, trails and the complete seeded random stream.
    pub fn configured(settings: &Settings) -> Self {
        let mut arena = Self::default();
        arena.colony.ants = (0..settings.agent_count)
            .map(|_| Ant {
                pos: arena.colony.home,
                carrying_food: false,
            })
            .collect();
        arena.rng = ChaCha8Rng::seed_from_u64(settings.random_seed);
        arena
    }
}
