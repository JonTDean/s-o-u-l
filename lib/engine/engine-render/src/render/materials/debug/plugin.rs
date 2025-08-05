//! Debug-materials **umbrella plug-in**
//!
//! This module wires together *all* run-time helpers that visualise debug
//! information in‐engine.  
//!
//! * Currently it registers [`DebugGridPlugin`], which draws an infinite,
//!   zoom-aware checker grid behind the world.  
//! * A stub for a `DebugFloorPlugin` is left in place (commented-out) so that
//!   future work can expand the overlay without changing the call-site that
//!   lives in `EngineRendererPlugin`.
//!
//! The plug-in is intentionally very small – it only aggregates sub-plug-ins
//! so that game crates can enable/disable the whole debug overlay with a
//! single `add_plugins(DebugMaterialsPlugin)` call.

use bevy::app::{App, Plugin};
// use tooling::debugging::floor::plugin::DebugFloorPlugin;
use crate::render::materials::debug::debug_grid::DebugGridPlugin;

/// Registers all debugging material plug-ins.
pub struct DebugMaterialsPlugin;

impl Plugin for DebugMaterialsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(DebugFloorPlugin);      // (future work)
        app.add_plugins(DebugGridPlugin);          // checker grid
    }
}
