//! Convenience re‑exports – downstream crates can simply
//!
//! ```rust
//! use models::prelude::*;
//! ```

pub use super::{
    automata::classical::plugin::ClassicalAutomataPlugin,
    automata::dynamical::plugin::DynamicalAutomataPlugin, plugin::ComputationalIntelligencePlugin,
};
