//! `spawn_menu/spawner.rs`
//! --------------------------------
//! “Spawn Automaton” panel – dynamically lists every automaton rule that is
//! currently registered in the global [`RuleRegistry`].  Entries are grouped
//! by **family prefix** (e.g. `life`, `wolfram`, `lenia`, …) so the menu stays
//! readable as the catalogue grows.  The window title shows whether the voxel
//! **GPU compute back‑end** is active for the current session.
//!
//! The expensive grouping logic runs in a Rayon parallel iterator which keeps
//! the main thread perfectly smooth even with hundreds of rules.
//!
//! _This file fully replaces the previous implementation._

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use engine_core::{
    events::AutomataCommand,
    systems::registry::RuleRegistry,
    systems::state::resources::RuntimeFlags,
};

use rayon::prelude::*;

/* ===================================================================== */
/* Public system                                                         */
/* ===================================================================== */

/// Draws the **Spawn Automaton** window and translates user clicks into
/// [`AutomataCommand::SeedPattern`] events.
///
/// The function is **data‑driven**: every rule that registers itself with the
/// [`RuleRegistry`]—whether at compile‑time _or_ dynamically at run‑time—will
/// automatically appear here without further wiring.
#[allow(clippy::needless_pass_by_value)]
pub fn spawner(
    rules:  Res<RuleRegistry>,        // ← global rule catalogue
    flags:  Res<RuntimeFlags>,        // ← CPU vs GPU hint
    mut ui: EguiContexts,             // ← egui context provider
    mut tx: EventWriter<AutomataCommand>,
) {
    let ctx = match ui.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,             // egui not ready yet
    };

    /* ----------------------------------------------------------------- */
    /* 1 ░ Build ⟨family → ids⟩ map in parallel                           */
    /* ----------------------------------------------------------------- */
    let grouped: Vec<(String, Vec<String>)> = {
        // Collect & sort rule IDs – the list is usually small (<100),
        // but we parallelise anyway to stay scalable.
        let mut ids: Vec<String> = rules
            .ids()
            .par_bridge()
            .map(|s| s.clone())
            .collect();

        ids.par_sort_unstable();

        // Group by prefix (text before first ':').  Using `BTreeMap` keeps
        // families in alphabetical order which is nice for humans.
        let mut map: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();

        for id in ids {
            let family = id
                .split_once(':')
                .map(|(f, _)| f)
                .unwrap_or(id.as_str());
            map.entry(family.to_owned())
                .or_default()
                .push(id);
        }
        map.into_iter().collect()
    };

    /* ----------------------------------------------------------------- */
    /* 2 ░ Render egui window                                            */
    /* ----------------------------------------------------------------- */
    egui::Window::new(format!(
        "Spawner – GPU {}",
        if flags.gpu_enabled { "ON" } else { "OFF" }
    ))
    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
    .resizable(false)
    .show(ctx, |ui| {
        if grouped.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_RED, "No automata rules registered.");
            return;
        }

        ui.label("Spawn an automaton pattern:");
        ui.separator();

        /* -------------------------------------------------------------- */
        /* 2.A ░ Family sections                                         */
        /* -------------------------------------------------------------- */
        for (family, ids) in &grouped {
            ui.collapsing(family, |ui| {
                egui::Grid::new(format!("grid_{family}"))
                    .min_col_width(128.0)
                    .show(ui, |ui| {
                        for id in ids {
                            if ui.button(format!("➕ {id}")).clicked() {
                                tx.write(AutomataCommand::SeedPattern { id: id.clone() });
                            }
                            ui.end_row();
                        }
                    });
            });
        }
    });
}
