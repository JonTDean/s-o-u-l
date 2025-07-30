//! Convenience re‑exports – downstream crates can simply
//!
//! ```rust
//! use models::prelude::*;
//! ```

pub use super::{
    plugin::ComputationalIntelligencePlugin,
    automata::classical::plugin::ClassicalAutomataPlugin,
    automata::dynamical::plugin::DynamicalAutomataPlugin,
};
