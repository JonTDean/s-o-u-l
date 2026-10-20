//! Per-channel zoom/pan and the content → screen fit used by CPU painters.
use bevy_egui::egui::{Pos2, Rect, Vec2, vec2};

/// Viewer-controlled zoom and pan for one CPU channel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageView {
    /// Scale factor relative to fit.
    pub zoom: f32,
    /// Offset in logical points.
    pub pan: Vec2,
}

impl Default for StageView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

impl StageView {
    /// Multiply zoom, clamped to 0.25–16×.
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.25, 16.0);
    }
}

/// Uniform scale and origin mapping content units to screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    /// Screen points per content unit.
    pub scale: f32,
    /// Screen position of content (0, 0).
    pub origin: Pos2,
    /// Content size in model units.
    pub content: Vec2,
}

impl Fit {
    /// Content coordinate → screen point.
    pub fn to_screen(&self, p: Vec2) -> Pos2 {
        self.origin + p * self.scale
    }

    /// Screen point → integer cell, if inside the content.
    pub fn to_cell(&self, p: Pos2) -> Option<(i32, i32)> {
        let c = (p - self.origin) / self.scale;
        let inside = c.x >= 0.0 && c.y >= 0.0 && c.x < self.content.x && c.y < self.content.y;
        inside.then_some((c.x as i32, c.y as i32))
    }
}

/// Fit `content` into 92 % of `rect`, then apply zoom about the centre and pan.
pub fn fit(content: Vec2, rect: Rect, view: StageView) -> Fit {
    let content = content.max(vec2(1.0, 1.0));
    let scale = (rect.width() / content.x).min(rect.height() / content.y) * 0.92 * view.zoom;
    let origin = rect.center() - content * scale * 0.5 + view.pan;
    Fit {
        scale,
        origin,
        content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::{Rect, pos2, vec2};

    #[test]
    fn content_is_centred_and_fitted() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 400.0));
        let f = fit(vec2(64.0, 64.0), rect, StageView::default());
        assert!((f.scale - 400.0 * 0.92 / 64.0).abs() < 1e-4);
        assert_eq!(f.to_screen(vec2(32.0, 32.0)), pos2(400.0, 200.0));
    }

    #[test]
    fn cursor_maps_back_to_cells() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(640.0, 640.0));
        let f = fit(vec2(64.0, 64.0), rect, StageView::default());
        assert_eq!(f.to_cell(f.to_screen(vec2(10.5, 3.5))), Some((10, 3)));
        assert_eq!(f.to_cell(pos2(-50.0, 0.0)), None);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut v = StageView::default();
        v.zoom_by(100.0);
        assert_eq!(v.zoom, 16.0);
        v.zoom_by(0.0001);
        assert_eq!(v.zoom, 0.25);
    }
}
