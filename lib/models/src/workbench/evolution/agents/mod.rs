//! Reproducible finite agent arenas; boids wrap and ants clamp at boundaries.
use crate::automata::dynamical::swarm::{
    ant_colony::{Ant, AntColony},
    boids::{Boid, step_boids},
};
use crate::workbench::{catalog::Kind, experiment::config::Settings};
use glam::{IVec3, Vec2};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// A bounded flock with a snapshot steering update.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flock {
    pub boids: Vec<Boid>,
}
impl Default for Flock {
    fn default() -> Self {
        let boids = (0..24)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 24.0;
                Boid {
                    position: Vec2::splat(32.0) + Vec2::from_angle(angle) * 18.0,
                    velocity: Vec2::from_angle(angle + 1.0),
                }
            })
            .collect();
        Self { boids }
    }
}
impl Flock {
    /// Advance one synchronous arena tick and enforce the finite domain.
    pub fn step(&mut self) {
        self.step_with_settings(&Settings::for_kind(Kind::Boids));
    }

    /// Apply flocking weights without replacing positions or velocities.
    pub fn step_with_settings(&mut self, settings: &Settings) {
        step_boids(
            &mut self.boids,
            settings.cohesion,
            settings.alignment,
            settings.separation,
            settings.separation_distance,
        );
        for boid in &mut self.boids {
            boid.position.x = boid.position.x.rem_euclid(64.0);
            boid.position.y = boid.position.y.rem_euclid(64.0);
        }
    }
}

/// Seeded foraging arena with persistent food sites and evaporating trails.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foraging {
    pub colony: AntColony,
    pub deliveries: u64,
    rng: ChaCha8Rng,
}
impl Default for Foraging {
    fn default() -> Self {
        let home = IVec3::new(32, 32, 0);
        Self {
            colony: AntColony {
                ants: (0..32)
                    .map(|_| Ant {
                        pos: home,
                        carrying_food: false,
                    })
                    .collect(),
                home,
                food_sources: vec![IVec3::new(38, 32, 0), IVec3::new(26, 32, 0)],
                pheromone: Default::default(),
            },
            deliveries: 0,
            rng: ChaCha8Rng::seed_from_u64(0x534f554c),
        }
    }
}
impl Foraging {
    /// Advance one synchronous arena tick and enforce the finite domain.
    pub fn step(&mut self) {
        self.step_with_settings(&Settings::for_kind(Kind::Ants));
    }

    /// Apply ant parameters while retaining trails, deliveries and RNG state.
    pub fn step_with_settings(&mut self, settings: &Settings) {
        let carrying: Vec<_> = self.colony.ants.iter().map(|a| a.carrying_food).collect();
        self.colony.step_configured(
            &mut self.rng,
            settings.evaporation,
            settings.deposit,
            settings.exploration,
        );
        for (ant, carried) in self.colony.ants.iter_mut().zip(carrying) {
            ant.pos = ant.pos.clamp(IVec3::ZERO, IVec3::new(63, 63, 0));
            if carried && !ant.carrying_food && ant.pos == self.colony.home {
                self.deliveries += 1;
            }
        }
        self.colony
            .pheromone
            .retain(|p, _| (0..64).contains(&p.x) && (0..64).contains(&p.y));
    }
}

#[cfg(test)]
mod tests;

mod configuration;
