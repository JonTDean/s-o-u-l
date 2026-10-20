//! Per-channel CPU stage view (zoom and pan).
use bevy_egui::egui::Vec2;

/// Zoom and pan of one CPU channel's painting.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StageView {
    /// Scale factor relative to fit.
    pub zoom: f32,
    /// Offset in logical points.
    pub pan: Vec2,
}
