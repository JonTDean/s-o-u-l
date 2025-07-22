//! Canonical frame-flow for S.O.U.L.
//!
//! ```
// !  Input (player / net / script)  ─►  Logic / Simulation  ─►  Render / UI
//! ```
//! The three `SystemSet`s establish an **unambiguous ordering** inside the
//! regular `Update` schedule.  Plugins just tag their systems with one of
//! the sets and Bevy guarantees correct sequencing.
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MainSet {
    Input,
    Logic,
    Render,
}