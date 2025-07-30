//! Computational‑Intelligence layer – pluggable rule‑sets + analytics.
//!
//! Add _one_ line in the main app to enable **all** default rule families:
//!
//! ```rust
//! app.add_plugins(models::ComputationalIntelligencePlugin);
//! ```
//!
//! *Later*: use cargo‑features to enable/disable sub‑families.

pub mod automata;
pub mod analytics;
pub mod prelude;
pub mod plugin;
