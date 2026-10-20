//! Elementary space-time history, Life cells and HPP density.
use super::{EMPTY_DOT, Fit, ramp_colour};
use crate::theme::tokens::color;
use bevy_egui::egui::{Painter, Rect, Vec2, vec2};
use engine_core::automata::channel::{Srgb, ramp};
use models::workbench::{catalog::Kind, grid::GridSession};

/// Paint one grid session; elementary rules show their 64-row history, time downward.
pub fn paint(painter: &Painter, fit: &Fit, grid: &GridSession, kind: Kind, colour: Srgb) {
    let width = grid.grid.size.x as usize;
    let cell = |x: usize, y: usize| {
        let min = fit.to_screen(vec2(x as f32, y as f32));
        Rect::from_min_size(min, Vec2::splat(fit.scale)).shrink(fit.scale * 0.08)
    };
    let dots = fit.scale >= 6.0;
    let stops = ramp(colour);
    let rows: Vec<Vec<u8>> = if grid.grid.size.y == 1 {
        grid.history.iter().cloned().collect()
    } else {
        grid.bytes().chunks(width).map(<[u8]>::to_vec).collect()
    };
    for (y, row) in rows.iter().enumerate() {
        for (x, value) in row.iter().enumerate() {
            match *value {
                0 if dots => {
                    painter.circle_filled(cell(x, y).center(), fit.scale * 0.06, EMPTY_DOT);
                }
                0 => {}
                // HPP cells hold direction bits; more particles read brighter.
                v if kind == Kind::Hpp => {
                    let t = 0.35 + 0.65 * v.count_ones() as f32 / 4.0;
                    painter.rect_filled(cell(x, y), fit.scale * 0.12, ramp_colour(stops, t));
                }
                _ => {
                    painter.rect_filled(cell(x, y), fit.scale * 0.12, color(colour));
                }
            }
        }
    }
}
