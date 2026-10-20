//! Tiny HUD panel (upper-left) showing every running automaton **plus one
//! debug button** that writes a 3 × 3 square into the atlas slice that belongs
//! to the *currently selected* automaton.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use engine_core::{
    events::{AutomatonId, DebugSeedSquare},
    prelude::AutomataRegistry,
};

/// Automaton highlighted in the HUD list.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct MinimapSelection(pub Option<AutomatonId>);

/* --------------------------------------------------------------------- */

fn friendly_name(id: &str) -> &str {
    match id {
        "wolfram:rule30" => "Wolfram Rule 30",
        "wolfram:rule110" => "Wolfram Rule 110",
        "life:conway" => "Conway’s Life",
        "life:dean" => "Dean’s Life",
        "lenia" => "Lenia (blob)",
        "lenia:orbium" => "Lenia (ring)",
        "particle:hpp" => "Lattice-gas HPP",
        _ => id,
    }
}

/* --------------------------------------------------------------------- */

pub fn show_active_automata(
    automata: Res<AutomataRegistry>,
    mut bench: ResMut<models::workbench::plugin::Workbench>,
    mut egui_ctx: EguiContexts<'_, '_>,
    mut sel: ResMut<MinimapSelection>,
    mut debug_tx: EventWriter<DebugSeedSquare>,
) {
    let Ok(ctx) = egui_ctx.ctx_mut() else { return };

    /* auto-clear the selection if the chosen automaton was removed */
    if let Some(id) = sel.0
        && automata.get(id).is_none()
    {
        sel.0 = None;
    }

    egui::Area::new("active_automata_panel".into())
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("Active automata:");
            if automata.list().is_empty() && bench.sessions.is_empty() {
                ui.label("(none)");
                sel.0 = None;
                return;
            }

            let mut selected = bench.selected;
            for (index, session) in bench.sessions.iter().enumerate() {
                if ui
                    .selectable_label(
                        index == selected,
                        format!("{} · {}", index + 1, session.title()),
                    )
                    .clicked()
                {
                    selected = index;
                }
            }
            bench.selected = selected;

            /* ----------------------------------------------------------------- */
            /* 1 ░ selection list                                                */
            /* ----------------------------------------------------------------- */
            for info in automata.list() {
                let is_selected = sel.0 == Some(info.id);
                let label = format!("• {}", friendly_name(&info.name));

                if ui.selectable_label(is_selected, label).clicked() {
                    sel.0 = if is_selected { None } else { Some(info.id) };
                }
            }

            /* ----------------------------------------------------------------- */
            /* 2 ░ debug helper – seed 3 × 3 square                              */
            /* ----------------------------------------------------------------- */
            ui.separator();
            if !automata.list().is_empty()
                && ui.button("📐 Debug • mark centre").clicked()
                && let Some(id) = sel.0
                && let Some(info) = automata.get(id)
            {
                debug_tx.write(DebugSeedSquare {
                    slice: info.slice.clone(), // ✅ matches new event field names
                    value: 255,
                });
            }
        });
}

pub mod workbench;
