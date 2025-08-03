//! Build & run the Bevy `App`.
//!
//! * Creates the core application skeleton (window / headless settings).
//! * Inserts global resources & the default [`AppState::MainMenu`].  
//! * Wires the canonical **Input → Logic → Render** pipeline for *both*
//!   variable-rate **`Update`** and deterministic **`FixedUpdate`**
//!   schedules.  
//! * Configures the fixed-step loop that emits [`SimulationStep`] events
//!   using the jitter-free [`Time<Fixed>`] source (see *PERF-001*).  
//! * Registers **all** core & feature plugins in the architecture-defined
//!   order.

use bevy::{
    prelude::*,
    window::WindowPlugin,
};

use engine_core::systems::{
    schedule::MainSet,
    state::AppState,
    simulation::{FixedStepConfig, SimAccumulator, accumulate_and_step, SimulationStep},
};
use tooling::debugging::plugin::DebugPlugin;

use crate::app::{
    builder::RuntimeConfig,
    plugin_registry::add_all_plugins,
    schedule,
};

/// Public one-liner used by `main()`.
pub fn run() {
    build(RuntimeConfig::load()).run();
}

/// Construct the fully-configured [`App`]; callers may `.run()` or add extras.
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

    /* 2 ░ global resources & always-on systems ----------------------- */
    app.init_state::<AppState>();           // default == MainMenu

    // Fixed-step configuration + accumulator + event type
    let fixed_cfg = FixedStepConfig::from_hz(cfg.simulation_rate_hz, cfg.max_sim_steps_per_frame);
    app.insert_resource(fixed_cfg)
        .init_resource::<SimAccumulator>()
        .add_event::<SimulationStep>();

    /* 3 ░ canonical schedule wiring --------------------------------- */
    schedule::configure(&mut app);

    // Jitter-free accumulator now runs in the deterministic tier.
    app.add_systems(
        FixedUpdate,
        accumulate_and_step
            .in_set(MainSet::Input)   // run before game-logic
    );


    /* 4 ░ register every feature / renderer plug-in ------------------ */
    add_all_plugins(&mut app);
    app.add_plugins(DebugPlugin);

    app
}
