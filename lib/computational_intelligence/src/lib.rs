//! Computational‑Intelligence layer – pluggable rule‑sets + analytics.
//!
//! Add _one_ line in the main app to enable **all** default rule families:
//!
//! ```rust
//! app.add_plugins(computational_intelligence::ComputationalIntelligencePlugin);
//! ```
//!
//! *Later*: use cargo‑features to enable/disable sub‑families.

pub mod automata;
pub mod analytics;
pub mod bridges;
pub mod registry;
pub mod prelude;
pub mod plugin;
