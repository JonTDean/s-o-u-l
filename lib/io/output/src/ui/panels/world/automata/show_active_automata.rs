//! HUD overlay that lists every automaton selected for the current scenario
//! and shows basic analytics in real time.
//
//! • Drawn in the top‑left corner while `AppState::InGame` is active.
//! • Each classical rule line shows the current agent count.
//! • Each dynamical rule line shows the instantaneous Φ (integration) value.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::ui::panels::main_menu::controller::scenario::new::ScenarioMeta;
use computational_intelligence::analytics::{
    iit_phi::compute_phi,
    swarm_metrics::swarm_summary,
};
use engine_core::core::world::World2D;

/// Translate registry IDs to human‑readable names.
fn friendly_name(id: &str) -> &str {
    match id {
        "wolfram:rule30"  => "Wolfram Rule 30",
        "wolfram:rule110" => "Wolfram Rule 110",
        "life:conway"     => "Conway’s Game of Life",
        "life:dean"       => "Dean’s Life",
        "lenia"           => "Lenia",
        _                 => id,
    }
}

/// Bevy system – registered by `AutomataPanelPlugin`.
pub fn show_active_automata(
    selected: Res<ScenarioMeta>,
    world:    Res<World2D>,
    mut egui_ctx: EguiContexts,
) {
    let ctx = egui_ctx
        .ctx_mut()
        .expect("Egui primary window context not found");

    /* ── analytics for this frame ─────────────────────────────────────── */
    let summary = swarm_summary(&world);
    let phi     = compute_phi(&world);

    egui::Area::new("active_automata_panel".into())
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing.y = 4.0;
            ui.label("Active Automata:");

            // Classical rules ------------------------------------------------
            for id in &selected.0.selected_classical {
                ui.label(format!("• {} – agents {}", friendly_name(id), summary.total_agents));
            }

            // Dynamical rules ------------------------------------------------
            for id in &selected.0.selected_dynamical {
                ui.label(format!("• {} – Φ {:.2}", friendly_name(id), phi));
            }

            if selected.0.selected_classical.is_empty()
                && selected.0.selected_dynamical.is_empty()
            {
                ui.label("(none)");
            }
        });
}
