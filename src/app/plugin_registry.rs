//! One-stop shop for registering every Bevy `Plugin`.
//
//! Centralising the list means:
//!   • A clean top-level `runner.rs` (no mile-long `.add_plugins()` chains).
//!   • Conditional compilation or run-time feature flags happen in *one*
//!     place (e.g. network server vs client vs disabled).
//
//! Down-stream code calls [`add_all_plugins()`] once; the order here
//! **must** match the architecture docs & Kanban cards.
//! One-stop shop for registering every Bevy `Plugin`.

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use bevy_egui::EguiPlugin;

use engine_core::{plugin::EngineCorePlugin, systems::state::StatePlugin};
use engine_render::plugin::EngineRendererPlugin;
use models::plugin::ComputationalIntelligencePlugin;
use ui::plugin::OutputPlugin;

/// Add **every** core & feature plugin in the required order.
pub fn add_all_plugins(app: &mut App) {
    // 0  egui framework (UI)
    app.add_plugins(EguiPlugin::default());

    /* 1 frame-time diagnostics */
    app.add_plugins(FrameTimeDiagnosticsPlugin::default()); 

    /* 2  dev / runtime state */
    app.add_plugins(StatePlugin);

    /* 3  core engine */
    app.add_plugins(EngineCorePlugin);

    /* 4  render stack */
    app.add_plugins(EngineRendererPlugin);

    /* 5  computational-intelligence layer */
    app.add_plugins(ComputationalIntelligencePlugin);

    /* 6  UI / HUD */
    app.add_plugins(OutputPlugin);
}

