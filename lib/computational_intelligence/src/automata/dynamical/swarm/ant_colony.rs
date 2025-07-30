//! Simplified ant colony simulation with pheromones (foraging behavior).

use bevy::math::IVec2;
use simulation_kernel::core::dim::{Dim, Dim2};
use rand::Rng;

pub struct Ant {
    pub pos: IVec2,
    pub carrying_food: bool,
}

pub struct AntColony {
    pub ants: Vec<Ant>,
    pub home: IVec2,
    pub food_sources: Vec<IVec2>,
    pub pheromone: std::collections::HashMap<IVec2, f32>,
}

impl AntColony {
    /// Advance the simulation by one time step.
    pub fn step(&mut self) {
        // Evaporate pheromone trail
        for value in self.pheromone.values_mut() {
            *value *= 0.99;
            if *value < 0.001 {
                *value = 0.0;
            }
        }
        // Move each ant
        for ant in &mut self.ants {
            if ant.carrying_food {
                // Ant with food heads towards home, dropping pheromone
                let dir = (self.home - ant.pos).clamp(IVec2::new(-1, -1), IVec2::new(1, 1));
                // Move one step toward home
                ant.pos += dir;
                // Drop pheromone at new position
                self.pheromone.entry(ant.pos).and_modify(|v| *v += 1.0).or_insert(1.0);
                // If reached home, drop the food
                if ant.pos == self.home {
                    ant.carrying_food = false;
                }
            } else {
                // Ant without food wanders or follows pheromone to find food
                let mut best_dir: Option<IVec2> = None;
                let mut best_pher = 0.0;
                // Look at neighboring cells for pheromone
                for off in Dim2::NEIGHBOUR_OFFSETS.iter() {
                    let npos = ant.pos + *off;
                    let pher = *self.pheromone.get(&npos).unwrap_or(&0.0);
                    if pher > best_pher {
                        best_pher = pher;
                        best_dir = Some(*off);
                    }
                }
                let next_dir = if let Some(dir) = best_dir {
                    // Follow strongest pheromone
                    dir
                } else {
                    // Random move if no pheromone signal
                    let directions = Dim2::NEIGHBOUR_OFFSETS;
                    directions[rand::rng().random_range(0..directions.len())]
                };
                ant.pos += next_dir;
                // Check for food at new position
                if self.food_sources.contains(&ant.pos) {
                    ant.carrying_food = true;
                    // (In a full simulation, we might decrease food source or mark it)
                }
            }
        }
    }
}
