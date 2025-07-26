//! Tiny HUD panel (upper‑left) that lists every running automaton plus a
//! very simple activity metric (live‑cell count).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use computational_intelligence::registry::AutomataRegistry;
use engine_core::core::cell::CellState;

/* --------------------------------------------------------------------- */

fn friendly_name(id: &str) -> &str {
    match id {
        "wolfram:rule30"  => "Wolfram Rule 30",
        "wolfram:rule110" => "Wolfram Rule 110",
        "life:conway"     => "Conway’s Life",
        "life:dean"       => "Dean’s Life",
        "lenia"           => "Lenia (blob)",
        "lenia:orbium"    => "Lenia – Orbium",
        "particle:hpp"    => "Lattice‑gas HPP",
        _                 => id,
    }
}

/* --------------------------------------------------------------------- */

pub fn show_active_automata(
    automata:  Res<AutomataRegistry>,
    mut egui_ctx: EguiContexts<'_, '_>,
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    egui::Area::new("active_automata_panel".into())
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(&ctx, |ui| {
            ui.heading("Active automata:");
            if automata.list().is_empty() {
                ui.label("(none)");
            }

            for info in automata.list() {
                let live = match &info.grid {
                    engine_core::engine::grid::GridBackend::Dense(g)  => {
                        g.cells.iter().filter(|c| !matches!(c.state, CellState::Dead)).count()
                    }
                    engine_core::engine::grid::GridBackend::Sparse(s) => {
                        s.iter().filter(|(_, c)| !matches!(c.state, CellState::Dead)).count()
                    }
                };
                ui.horizontal(|ui| {
                    ui.label(format!("• {} – {live} live", friendly_name(&info.name)));
                });
            }
        });
}