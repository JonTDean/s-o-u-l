//! Scheduling utilities and long-lived application systems.

pub mod execution;
pub mod registry;
pub use execution::{schedule, simulation};
pub mod state;

// Fixed-step accumulation is exported through execution::simulation.
