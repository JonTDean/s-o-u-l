//! Canonical frame-flow for S.O.U.L.
//!
//! ```
// !  Input (player / net / script)  ─►  Logic / Simulation  ─►  Render / UI
//! ```
//! The three `SystemSet`s establish an **unambiguous ordering** inside the
//! regular `Update` schedule.  Plugins just tag their systems with one of
//! the sets and Bevy guarantees correct sequencing.
use bevy::prelude::*;

/// Canonical ordering for engine systems within Bevy's `Update` schedule.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MainSet {
    /// User input and external events.
    Input,
    /// Deterministic simulation and game logic.
    Logic,
    /// Rendering and UI work.
    Render,
}
