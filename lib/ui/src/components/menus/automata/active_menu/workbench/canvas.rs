//! Grid, space-time and agent views share a fixed 64-unit arena.
use bevy_egui::egui::{self, Color32, Rect, Stroke, Vec2};
use models::workbench::session::Model;

pub(super) fn draw(ui: &mut egui::Ui, model: &Model) {
    let side = ui.available_width().clamp(128.0, 360.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(side), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 15, 22));
    let point = |x: f32, y: f32| rect.min + Vec2::new(x, y) * (side / 64.0);
    let cell = |x: usize, y: usize, color| {
        painter.rect_filled(
            Rect::from_min_size(point(x as f32, y as f32), Vec2::splat(side / 64.0)),
            0.0,
            color,
        );
    };
    let cyan = Color32::from_rgb(80, 210, 205);
    match model {
        Model::Grid(grid) => {
            if grid.grid.size.y == 1 {
                for (y, row) in grid.history.iter().enumerate() {
                    for (x, value) in row.iter().enumerate() {
                        if *value != 0 {
                            cell(x, y, cyan);
                        }
                    }
                }
            } else {
                for (index, value) in grid.bytes().iter().enumerate() {
                    if *value != 0 {
                        let shade = if *value == 255 {
                            cyan
                        } else {
                            Color32::from_rgb(60, 90 + value.count_ones() as u8 * 40, 200)
                        };
                        cell(index % 64, index / 64, shade);
                    }
                }
            }
        }
        Model::Boids(flock) => {
            for boid in &flock.boids {
                let p = point(boid.position.x, boid.position.y);
                let velocity = boid.velocity.normalize_or_zero();
                painter.circle_filled(p, 2.5, cyan);
                painter.line_segment(
                    [p, p + Vec2::new(velocity.x, velocity.y) * 9.0],
                    Stroke::new(1.5, cyan),
                );
            }
        }
        Model::Ants(arena) => {
            for (p, strength) in &arena.colony.pheromone {
                let green = (strength * 30.0).clamp(20.0, 180.0) as u8;
                cell(p.x as usize, p.y as usize, Color32::from_rgb(20, green, 50));
            }
            for food in &arena.colony.food_sources {
                painter.circle_filled(
                    point(food.x as f32, food.y as f32),
                    6.0,
                    Color32::LIGHT_GREEN,
                );
            }
            let home = arena.colony.home;
            painter.circle_stroke(
                point(home.x as f32, home.y as f32),
                7.0,
                Stroke::new(2.0, Color32::WHITE),
            );
            for ant in &arena.colony.ants {
                painter.circle_filled(
                    point(ant.pos.x as f32, ant.pos.y as f32),
                    2.5,
                    if ant.carrying_food {
                        Color32::YELLOW
                    } else {
                        cyan
                    },
                );
            }
            ui.label(format!(
                "Food deliveries: {} · yellow ants carry food",
                arena.deliveries
            ));
        }
        Model::Symbolic(_) => {}
    }
}
