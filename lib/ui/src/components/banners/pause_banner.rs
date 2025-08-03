// lib/ui/src/overlays/pause_banner.rs
//! Translucent "Paused" banner overlay (UI-001).
//!
//! * Shows whenever the **root [`AppState`] is `Paused`**.
//! * Renders a semi-transparent banner across the top-centre of the screen
//!   with the ⏸ “Paused” glyph and hot-key hints:
//!
//! ```text
//! ⏸  Simulation paused   •   P = resume   N = step
//! ```
//!
//! ### Thread-safety
//! All state is read-only.  The system is therefore **data-parallel** and
//! may run on any Bevy thread without synchronisation.
//!
//! ### Styling
//! * Background: `rgba(0,0,0,180)` (≈ 70 % opacity) for unobtrusive contrast.
//! * Text colour: `Color32::LIGHT_GREEN` — matches debug-overlay palette.
//! * Padding & margins follow the global `styles::BOTTOM_PAD` constant.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::prelude::AppState;

/// Bevy *plugin* that registers the pause-banner UI system.
#[derive(Default)]
pub struct PauseBannerPlugin;
impl Plugin for PauseBannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            pause_banner_ui.run_if(in_state(AppState::Paused)),
        );
    }
}

/// Renders the translucent *Paused* banner every frame **while paused**.
fn pause_banner_ui(mut egui_ctx: EguiContexts<'_, '_>) {
    // ── 1 ░ egui context ────────────────────────────────────────────
    let Ok(ctx) = egui_ctx.ctx_mut() else { return }; // current window only

    // ── 2 ░ banner geometry ────────────────────────────────────────
    const HEIGHT: f32 = 42.0;

    egui::Area::new("pause_banner".into())
        .anchor(egui::Align2::CENTER_TOP, [0.0, 16.0])
        .interactable(false)
        .show(ctx, |ui| {
            use egui::{Color32, Frame, RichText, Margin};

            Frame::new()
                .fill(Color32::from_rgba_premultiplied(0, 0, 0, 180))
                .corner_radius(6.0)
                .stroke(egui::Stroke::NONE)
                .outer_margin(Margin {
                    left: 12,
                    right: 12,
                    top: 0,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    ui.set_height(HEIGHT);
                    ui.horizontal_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new("⏸ Simulation paused").strong());
                        ui.add_space(12.0);
                        ui.label(RichText::new("P = resume   N = step").color(Color32::LIGHT_GREEN));
                        ui.add_space(4.0);
                    });
                });
        });
}
