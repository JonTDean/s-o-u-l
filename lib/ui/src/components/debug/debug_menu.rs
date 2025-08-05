//! ui/components/debug/debug_menu.rs
//! ---------------------------------------------------------------------------
//! Debug overlay (F3) – now the **single home** for *all* runtime gizmo flags.
//!
//! * Moved the old “Toggle Debug Grid” button from the _Spawn one pattern_
//!   window into this menu.  The redundant control has been removed from the
//!   other overlay, so there’s only one source of truth.
//! * Renamed entries for clarity:
//!     * "Floor grid" → grey XY plane gizmo (`FLOOR_GRID`)
//!     * "3‑D voxel grid" → coloured XYZ voxel lattice (`GRID_3D`)
//!
//! © 2025 Obaven Inc.

use bevy::{
    ecs::system::{Local, Res},
    input::{keyboard::KeyCode, ButtonInput},
};
use bevy_egui::{egui, EguiContexts};

/// Immediate‑mode egui window toggled with **F3**.
pub fn debug_menu(
    mut egui_ctx: EguiContexts,
    keys:        Res<ButtonInput<KeyCode>>,
    mut open:    Local<bool>,
) {
    // Hot‑key – open/close.
    if keys.just_pressed(KeyCode::F3) {
        *open = !*open;
    }
    if !*open {
        return;
    }

    let ctx = egui_ctx.ctx_mut().expect("Egui context unavailable");

    egui::Window::new("Debug Menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Hot‑keys:  F3 show/hide  •  Ctrl+Click resets all");
        });
        ui.separator();

    });
}
