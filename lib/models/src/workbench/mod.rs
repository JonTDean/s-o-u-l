//! Runnable automata catalog and isolated, bounded CPU workbench sessions.
pub mod evolution;
pub use evolution::{agents, grid, symbolic};
pub mod catalog;
pub mod experiment;
pub mod plugin;
pub mod session;
