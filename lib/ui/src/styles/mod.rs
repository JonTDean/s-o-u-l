//! This module defines shared UI style constants and helper functions for styling UI elements.
pub mod fade;
pub mod plugin;

use bevy_egui::egui;

/// Re-usable constant for bottom padding in UI layouts.
pub const BOTTOM_PAD: f32 = 20.0;

/// Returns a frame style that paints the entire background a uniform dark blue.
pub fn fullscreen_bg() -> egui::Frame {
    egui::Frame {
        fill: egui::Color32::from_rgb(40, 40, 60),  // dark blue
        ..Default::default()
    }
}

/// Returns a frame style for side panels with a slightly lighter dark blue color.
pub fn panel_bg() -> egui::Frame {
    egui::Frame {
        fill: egui::Color32::from_rgb(50, 50, 75),  // lighter variant for side panel
        ..Default::default()
    }
}
