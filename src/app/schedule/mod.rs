//! `app::schedule` – centralised schedule wiring helpers.
//!
//! This module injects the canonical **Input → Logic → Render** [`MainSet`]
//! ordering into *both* Bevy’s variable‑rate **`Update`** schedule *and* the
//! deterministic **`FixedUpdate`** schedule.  Keeping the configuration in
//! one place avoids copy‑paste across binaries and tests.
//!
//! Down‑stream code (e.g. [`crate::runner`]) should call [`configure`] once
//! right after adding Bevy’s `DefaultPlugins`.

use bevy::prelude::*;

/// Three‑band pipeline shared with `engine‑core`.
pub use engine_core::systems::schedule::MainSet;

/* ====================================================================== */
/* Public API                                                             */
/* ====================================================================== */

/// Inserts `MainSet::{Input, Logic, Render}` into **both** `Update` and
/// `FixedUpdate` schedules.
#[inline]
pub fn configure(app: &mut App) {
    // ── variable‑rate schedule (render framerate) ────────────────────────
    app.configure_sets(
        Update,
        (
            MainSet::Input,
            MainSet::Logic.after(MainSet::Input),
            MainSet::Render.after(MainSet::Logic),
        ),
    );

    // ── fixed‑timestep schedule (deterministic) ─────────────────────────
    app.configure_sets(
        FixedUpdate,
        (
            MainSet::Input,
            MainSet::Logic.after(MainSet::Input),
            MainSet::Render.after(MainSet::Logic),
        ),
    );
}
