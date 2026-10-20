//! Model painters for CPU channels; every colour derives from the channel colour.
use super::view::Fit;
use bevy_egui::egui::{Color32, Painter, Vec2, vec2};
use engine_core::automata::channel::Srgb;
use models::workbench::{catalog::Kind, session::Model};
pub mod agents;
pub mod grid;
pub mod symbolic;

/// Faint marker for empty cells when cells are large enough to show it.
pub const EMPTY_DOT: Color32 = Color32::from_rgb(0x2c, 0x2b, 0x27);

/// Piecewise-linear sample of a four-stop ramp at `t` ∈ [0, 1]; stops sit at thirds.
pub fn ramp_colour(stops: [Srgb; 4], t: f32) -> Color32 {
    let x = t.clamp(0.0, 1.0) * 3.0;
    let i = (x.floor() as usize).min(2);
    let f = x - i as f32;
    let (a, b) = (stops[i].0, stops[i + 1].0);
    // Integer endpoints make f = 0 and f = 1 reproduce the stops exactly.
    let lerp = |k: usize| (f32::from(a[k]) + (f32::from(b[k]) - f32::from(a[k])) * f).round() as u8;
    Color32::from_rgb(lerp(0), lerp(1), lerp(2))
}

/// Content size in model units for fitting.
pub fn content_size(model: &Model) -> Vec2 {
    match model {
        Model::Grid(g) if g.grid.size.y == 1 => vec2(g.grid.size.x as f32, 64.0),
        Model::Grid(g) => vec2(g.grid.size.x as f32, g.grid.size.y as f32),
        Model::Symbolic(m) => symbolic::extent(m),
        Model::Boids(_) | Model::Ants(_) => vec2(64.0, 64.0),
    }
}

/// Paint `model` into `painter` with `fit` in the channel colour.
pub fn paint(painter: &Painter, fit: &Fit, model: &Model, kind: Kind, colour: Srgb) {
    match model {
        Model::Grid(g) => grid::paint(painter, fit, g, kind, colour),
        Model::Symbolic(m) => symbolic::paint(painter, fit, m, colour),
        Model::Boids(f) => agents::boids(painter, fit, f, colour),
        Model::Ants(a) => agents::ants(painter, fit, a, colour),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::tokens::color;
    use engine_core::automata::channel::{SIGNAL, ramp};

    #[test]
    fn ramp_colour_hits_stops_at_thirds() {
        let stops = ramp(SIGNAL);
        assert_eq!(ramp_colour(stops, 0.0), color(stops[0]));
        assert_eq!(ramp_colour(stops, 2.0 / 3.0), color(stops[2]));
        assert_eq!(ramp_colour(stops, 1.0), color(stops[3]));
        assert_eq!(ramp_colour(stops, 7.0), color(stops[3]), "clamped");
    }
}
