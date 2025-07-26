//! Convenience re‑exports – downstream crates can simply
//!
//! ```rust
//! use computational_intelligence::prelude::*;
//! ```

pub use super::{
    plugin::ComputationalIntelligencePlugin,
    automata::classical::plugin::ClassicalAutomataPlugin,
    automata::dynamical::plugin::DynamicalAutomataPlugin,
    registry::{RuleRegistry, AutomataRegistry, AutomatonInfo},
};
