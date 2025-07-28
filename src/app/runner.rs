//! Build & run the Bevy `App`.
//
//! * Creates the core `App` skeleton (window / headless settings).
//! * Inserts global resources & default [`AppState::MainMenu`].
//! * Defines the canonical three-stage update schedule.
//! * Registers **all** feature plugins in architecture-defined order.

use bevy::{
    prelude::*,
    window::WindowPlugin,
};
use engine_core::schedule::MainSet;

use crate::app::{
    builder::RuntimeConfig,
    plugin_registry::{add_all_plugins, PluginFlags},
};

/// Public one-liner used by `main()`.
pub fn run() { build(RuntimeConfig::load()).run(); }

/// Construct the fully-configured [`App`]; callers may `.run()` or add extras.
pub fn build(cfg: RuntimeConfig) -> App {
    /* ── 1. Core `App` skeleton ────────────────────────────────────────── */
    let mut app = App::new();

    if cfg.headless {
        app.add_plugins(DefaultPlugins.build().disable::<WindowPlugin>());
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title:       "S.O.U.L. – Swarm Orchestrator for Autonomous Learners".into(),
                resolution:  (1_280., 720.).into(),
                resizable:   true,
                ..default()
            }),
            ..default()
        }));
    }

    /* ── 2. Global resources & always-on utility systems ───────────────── */
    app.init_state::<engine_core::state::AppState>();  // ← default == MainMenu ✅

    /* ── 3. Canonical 3-phase update schedule ──────────────────────────── */
    app.configure_sets(
        Update,
        (
            MainSet::Input,
            MainSet::Logic.after(MainSet::Input),
            MainSet::Render.after(MainSet::Logic),
        ),
    );

    /* ── 4. Register every feature / renderer plugin in one place ─────── */
    add_all_plugins(
        &mut app,
        PluginFlags { networking: &cfg.networking },
    );

    app
}
