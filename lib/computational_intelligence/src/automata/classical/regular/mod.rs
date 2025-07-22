//! Type‑3 regular automata: finite‑state machines, 1D cellular automata, etc.
//!
//! Contains implementations for Wolfram 1‑D CA and regex/NFA examples.

pub mod wolfram_1d;
pub mod regex_nfa;
pub mod plugin;

// Re-export the plugin for external use.
pub use plugin::RegularAutomataPlugin;
