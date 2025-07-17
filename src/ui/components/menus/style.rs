use bevy_egui::egui;

/// Uniform dark-blue frame used by every modal window.
pub fn window_frame() -> egui::Frame {
    egui::Frame {
        fill: egui::Color32::from_rgb(40, 40, 60),
        // `f32` → `Margin` and `CornerRadius` via `Into`
        inner_margin: 8.0.into(),
        corner_radius: 6.0.into(),
        ..Default::default()
    }
}
