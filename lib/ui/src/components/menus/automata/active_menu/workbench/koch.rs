//! Turtle interpretation of the Koch production with sixty-degree turns.
use bevy_egui::egui::{self, Color32, Stroke, Vec2};

pub(super) fn draw(ui: &mut egui::Ui, word: &str) {
    let mut points = vec![Vec2::ZERO];
    let mut angle = 0.0_f32;
    for symbol in word.chars() {
        match symbol {
            'F' => points.push(*points.last().unwrap() + Vec2::angled(angle)),
            '+' => angle += std::f32::consts::PI / 3.0,
            '-' => angle -= std::f32::consts::PI / 3.0,
            _ => {}
        }
    }
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 240.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 15, 22));
    let min = points
        .iter()
        .fold(Vec2::splat(f32::INFINITY), |a, b| a.min(*b));
    let max = points
        .iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), |a, b| a.max(*b));
    let span = (max - min).max(Vec2::splat(1.0));
    let scale = ((rect.width() - 20.0) / span.x).min((rect.height() - 20.0) / span.y);
    let origin = rect.center() - span * scale / 2.0;
    for segment in points.windows(2) {
        painter.line_segment(
            [
                origin + (segment[0] - min) * scale,
                origin + (segment[1] - min) * scale,
            ],
            Stroke::new(1.5, Color32::from_rgb(80, 210, 205)),
        );
    }
}
