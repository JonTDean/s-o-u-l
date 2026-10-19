//! Simplified ant-colony example (XY motion, z = 0).

use glam::IVec3;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct Ant {
    pub pos: IVec3,
    pub carrying_food: bool,
}

pub struct AntColony {
    pub ants: Vec<Ant>,
    pub home: IVec3,
    pub food_sources: Vec<IVec3>,
    pub pheromone: std::collections::HashMap<IVec3, f32>,
}

impl AntColony {
    pub fn step(&mut self) {
        self.step_with_rng(&mut rand::rng());
    }

    /// Inject randomness so reset and numerical verification can reproduce a run.
    pub fn step_with_rng(&mut self, rng: &mut impl Rng) {
        self.step_configured(rng, 0.01, 1.0, 0.0);
    }

    /// Apply evaporation, deposition and exploration to a reproducible tick.
    pub fn step_configured(
        &mut self,
        rng: &mut impl Rng,
        evaporation: f32,
        deposit: f32,
        exploration: f32,
    ) {
        /* evaporate pheromones */
        for v in self.pheromone.values_mut() {
            *v *= 1.0 - evaporation;
            if *v < 0.001 {
                *v = 0.0;
            }
        }

        self.pheromone.retain(|_, value| *value > 0.0);

        /* neighbour offsets in XY plane */
        const DIRS: [IVec3; 8] = [
            IVec3::new(-1, -1, 0),
            IVec3::new(0, -1, 0),
            IVec3::new(1, -1, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 1, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(1, 1, 0),
        ];

        for ant in &mut self.ants {
            if ant.carrying_food {
                let dir = (self.home - ant.pos).clamp(IVec3::new(-1, -1, 0), IVec3::new(1, 1, 0));
                ant.pos += dir;
                self.pheromone
                    .entry(ant.pos)
                    .and_modify(|v| *v += deposit)
                    .or_insert(deposit);
                if ant.pos == self.home {
                    ant.carrying_food = false;
                }
            } else {
                let mut best_dir = None;
                let mut best_pher = 0.0;
                for off in DIRS {
                    let npos = ant.pos + off;
                    let pher = *self.pheromone.get(&npos).unwrap_or(&0.0);
                    if pher > best_pher {
                        best_pher = pher;
                        best_dir = Some(off);
                    }
                }
                if exploration > 0.0 && rng.random_bool(exploration as f64) {
                    best_dir = None;
                }
                let chosen = best_dir.unwrap_or_else(|| DIRS[rng.random_range(0..DIRS.len())]);
                ant.pos += chosen;
                if self.food_sources.contains(&ant.pos) {
                    ant.carrying_food = true;
                }
            }
        }
    }
}
