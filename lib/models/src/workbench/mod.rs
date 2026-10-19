//! Runnable automata catalog and isolated, bounded CPU workbench sessions.
pub mod evolution;
pub use evolution::{agents, grid, symbolic};
pub mod catalog;
pub mod operations;
pub use operations::{experiment, project};
pub mod plugin;
pub mod session;
