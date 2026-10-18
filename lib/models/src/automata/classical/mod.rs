//! Chomsky‑aligned symbolic automata (Types 3 → 0).

pub mod regular;

pub mod grammars;
pub use grammars::{contextful, contextless};
pub mod plugin;
pub mod turing;
