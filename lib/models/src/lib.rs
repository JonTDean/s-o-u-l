//! Computational‑Intelligence layer – pluggable rule‑sets + analytics.
//!
//! Add _one_ line in the main app to enable **all** default rule families:
//!
//! ```rust
//! # let mut app = bevy::app::App::new();
//! app.add_plugins(models::ComputationalIntelligencePlugin);
//! ```
//!
//! *Later*: use cargo‑features to enable/disable sub‑families.

pub mod analytics;
pub mod automata;
pub mod plugin;
pub mod prelude;

/// Root entry point for registering the automaton families and rule metadata.
pub use plugin::ComputationalIntelligencePlugin;

pub mod workbench;
