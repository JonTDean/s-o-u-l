//! Boids as heading triangles; ants with food, nest and pheromone ramp.
use super::{Fit, ramp_colour};
use crate::theme::tokens::*;
use bevy_egui::egui::{Painter, Rect, Shape, Stroke, Vec2, vec2};
use engine_core::automata::channel::{Srgb, ramp};
use models::workbench::agents::{Flock, Foraging};

/// Paint a flock.
pub fn boids(painter: &Painter, fit: &Fit, flock: &Flock, colour: Srgb) {
    for boid in &flock.boids {
        let heading = boid.velocity.normalize_or_zero();
        let (h, side) = (vec2(heading.x, heading.y), vec2(-heading.y, heading.x));
        let p = vec2(boid.position.x, boid.position.y);
        let points = [
            p + h * 1.2,
            p - h * 0.8 + side * 0.65,
            p - h * 0.4,
            p - h * 0.8 - side * 0.65,
        ]
        .map(|q| fit.to_screen(q));
        painter.add(Shape::convex_polygon(
            points.to_vec(),
            color(colour),
            Stroke::NONE,
        ));
    }
}

/// Paint a foraging arena.
pub fn ants(painter: &Painter, fit: &Fit, arena: &Foraging, colour: Srgb) {
    let stops = ramp(colour);
    let centre = |x: i32, y: i32| fit.to_screen(vec2(x as f32 + 0.5, y as f32 + 0.5));
    for (p, strength) in &arena.colony.pheromone {
        let min = fit.to_screen(vec2(p.x as f32, p.y as f32));
        let rect = Rect::from_min_size(min, Vec2::splat(fit.scale));
        painter.rect_filled(
            rect,
            0.0,
            ramp_colour(stops, (strength / 6.0).clamp(0.05, 0.6)),
        );
    }
    for food in &arena.colony.food_sources {
        painter.circle_filled(centre(food.x, food.y), fit.scale * 0.9, OK);
    }
    let home = arena.colony.home;
    painter.circle_stroke(centre(home.x, home.y), fit.scale, Stroke::new(2.0, INK_HI));
    for ant in &arena.colony.ants {
        let c = if ant.carrying_food {
            color(stops[3])
        } else {
            color(colour)
        };
        painter.circle_filled(centre(ant.pos.x, ant.pos.y), fit.scale * 0.35, c);
    }
}
