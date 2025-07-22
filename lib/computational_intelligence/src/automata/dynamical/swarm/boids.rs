//! Boids flocking algorithm implementation.

use bevy::prelude::Vec2;

pub struct Boid {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Boid {
    pub fn new(x: f32, y: f32) -> Self {
        Boid {
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
        }
    }
}

/// Advances the boids simulation by one step using standard flocking rules.
pub fn step_boids(boids: &mut [Boid], cohesion_weight: f32, alignment_weight: f32, separation_weight: f32, separation_distance: f32) {
    let n = boids.len();
    if n == 0 { return; }
    // Make a copy of current velocities to use in calculations
    let current_vel: Vec<Vec2> = boids.iter().map(|b| b.velocity).collect();
    for i in 0..n {
        let mut center = Vec2::ZERO;
        let mut avg_vel = Vec2::ZERO;
        let mut count = 0;
        let mut separation = Vec2::ZERO;
        for j in 0..n {
            if i == j { continue; }
            let diff = boids[j].position - boids[i].position;
            let dist_sq = diff.length_squared();
            let vision_range = 100.0; // neighborhood radius
            if dist_sq < vision_range * vision_range {
                center += boids[j].position;
                avg_vel += current_vel[j];
                count += 1;
            }
            if dist_sq < separation_distance * separation_distance && dist_sq > 0.0 {
                // steer away from too-close boid j
                separation -= diff.normalize_or_zero() / diff.length();
            }
        }
        if count > 0 {
            center /= count as f32;
            avg_vel /= count as f32;
            // Cohesion: steer towards center of neighbors
            boids[i].velocity += (center - boids[i].position) * cohesion_weight;
            // Alignment: adjust velocity towards average velocity of neighbors
            boids[i].velocity += (avg_vel - boids[i].velocity) * alignment_weight;
        }
        // Separation: avoid crowding
        boids[i].velocity += separation * separation_weight;
        // Limit speed to a maximum
        let max_speed = 5.0;
        if boids[i].velocity.length() > max_speed {
            boids[i].velocity = boids[i].velocity.normalize() * max_speed;
        }
    }
    // Update positions
    for boid in boids.iter_mut() {
        boid.position += boid.velocity;
    }
}
