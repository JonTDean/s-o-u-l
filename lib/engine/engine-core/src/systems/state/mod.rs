//! **High-level application state** hub.
//!
//! Re-exports the [`AppState`] enum and the [`StatePlugin`] so downstream
//! crates only need a single `use crate::state::*;` line.

/// Enum listing every top-level state (main menu, in-game, …).
pub mod app_state;

/// Plug-in that initialises long-lived state resources.
pub mod plugin;

/// Persistent settings, session counters, runtime feature flags.
pub mod resources;

pub use app_state::AppState;
pub use plugin::StatePlugin;