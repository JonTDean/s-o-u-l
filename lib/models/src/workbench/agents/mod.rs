//! Reproducible finite agent arenas; boids wrap and ants reflect at boundaries.
use crate::automata::dynamical::swarm::{
    ant_colony::{Ant, AntColony},
    boids::{Boid, step_boids},
};
use glam::{IVec3, Vec2};
use rand::{SeedableRng, rngs::StdRng};

/// A bounded flock with a snapshot steering update.
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
        step_boids(&mut self.boids, 0.002, 0.08, 0.1, 4.0);
        for boid in &mut self.boids {
            boid.position.x = boid.position.x.rem_euclid(64.0);
            boid.position.y = boid.position.y.rem_euclid(64.0);
        }
    }
}

/// Seeded foraging arena with persistent food sites and evaporating trails.
pub struct Foraging {
    pub colony: AntColony,
    pub deliveries: u64,
    rng: StdRng,
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
            rng: StdRng::seed_from_u64(0x534f554c),
        }
    }
}
impl Foraging {
    /// Advance one synchronous arena tick and enforce the finite domain.
    pub fn step(&mut self) {
        let carrying: Vec<_> = self.colony.ants.iter().map(|a| a.carrying_food).collect();
        self.colony.step_with_rng(&mut self.rng);
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
