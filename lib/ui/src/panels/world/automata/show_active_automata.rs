//! Tiny HUD panel (upper‑left) that lists every running automaton plus a
//! very simple activity metric (live‑cell count).  Clicking a line makes
//! that automaton the *current minimap target*.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use simulation_kernel::{core::cell::CellState, grid::GridBackend};
use engine::systems::registry::AutomataRegistry;

use crate::panels::world::minimap_overlay::MinimapSelection;

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
    automata:    Res<AutomataRegistry>,
    mut egui_ctx: EguiContexts<'_, '_>,
    mut sel:      ResMut<MinimapSelection>,
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    /* auto‑clear selection if the chosen automaton was removed */
    if let Some(id) = sel.0 {
        if automata.get(id).is_none() {
            sel.0 = None;
        }
    }

    egui::Area::new("active_automata_panel".into())
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(&ctx, |ui| {
            ui.heading("Active automata:");
            if automata.list().is_empty() {
                ui.label("(none)");
                sel.0 = None;
                return;
            }

            for info in automata.list() {
                let live = match &info.grid {
                    GridBackend::Dense(g)  => {
                        g.cells.iter().filter(|c| !matches!(c.state, CellState::Dead)).count()
                    }
                    GridBackend::Sparse(s) => {
                        s.iter().filter(|(_, c)| !matches!(c.state, CellState::Dead)).count()
                    }
                };

                let is_selected = sel.0 == Some(info.id);
                let label = format!("• {} – {live} live", friendly_name(&info.name));

                if ui.selectable_label(is_selected, label).clicked() {
                    // toggle selection: click again to clear
                    sel.0 = if is_selected { None } else { Some(info.id) };
                }
            }
        });
}
