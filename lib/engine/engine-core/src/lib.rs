//! Core ECS resources and runtime logic shared across the engine.
//!
//! This crate hosts low-level building blocks used by rendering, GPU compute
//! and higher level gameplay systems.  Everything exposed here is intended to
//! be stable for consumers within the workspace.

#![warn(missing_docs)]

pub mod prelude;

pub mod automata;
pub mod systems;
pub mod world;

pub mod events;
pub mod plugin;
