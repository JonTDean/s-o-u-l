//! Scenario sub-scenes (*New* & *Load*).
//!
//! Kept in a standalone module so the public `ScenariosPlugin` can be pulled
//! in by the root scene router with a single line.

pub mod load;
pub mod new;
pub mod plugin;

pub use plugin::ScenariosPlugin;
