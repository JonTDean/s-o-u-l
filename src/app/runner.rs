//! Build & run the Bevy `App`.
//
//! * Creates the core `App` skeleton (window / headless settings).
//! * Inserts global resources & default [`AppState::MainMenu`].
//! * Defines the canonical three‑stage update schedule **plus** the fixed‑step
//!   accumulator → [`SimulationStep`] event emitter.
//! * Registers **all** feature plug-ins in architecture-defined order.
//! 

use std::time::Duration;

use bevy::{
    prelude::*,
    window::WindowPlugin,
};

use engine_core::systems::{
    schedule::MainSet,
    state::AppState,
    simulation::{FixedStepConfig, SimAccumulator, accumulate_and_step, SimulationStep},
};
use engine_render::debug_plugin::DebugPlugin;

use crate::app::{
    builder::RuntimeConfig,
    plugin_registry::add_all_plugins,
};

/// Public one‑liner used by `main()`.
pub fn run() {
    build(RuntimeConfig::load()).run();
}

/// Construct the fully‑configured [`App`]; callers may `.run()` or add extras.
pub fn build(cfg: RuntimeConfig) -> App {
    /* 1 ░ core skeleton --------------------------------------------- */
    let mut app = App::new();

    if cfg.headless {
        app.add_plugins(DefaultPlugins.build().disable::<WindowPlugin>());
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title:      "S.O.U.L. – Swarm Orchestrator for Autonomous Learners".into(),
                resolution: (1_280., 720.).into(),
                resizable:  true,
                ..default()
            }),
            ..default()
        }));
    }

    /* 2 ░ global resources & always‑on systems ----------------------- */
    app.init_state::<AppState>();           // default == MainMenu

    // Fixed‑step loop configuration + accumulator + event type
    let fixed_cfg = FixedStepConfig::from_hz(cfg.simulation_rate_hz, cfg.max_sim_steps_per_frame);
    app.insert_resource(fixed_cfg)
        .init_resource::<SimAccumulator>()
        .add_event::<SimulationStep>()
        // Run once per *render* frame to translate real‑time into 0‑N steps.
        .add_systems(Update, accumulate_and_step.before(MainSet::Logic));

    /* 3 ░ canonical 3‑phase update schedule -------------------------- */
    app.configure_sets(
        Update,
        (
            MainSet::Input,
            MainSet::Logic.after(MainSet::Input),
            MainSet::Render.after(MainSet::Logic),
        ),
    );

    app.add_plugins(DebugPlugin);

    /* 4 ░ register every feature / renderer plug‑in ------------------ */
    add_all_plugins(&mut app);

    app
}
