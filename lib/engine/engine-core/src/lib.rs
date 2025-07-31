//! Dimension-agnostic engine core for S.O.U.L.
//!
//! This crate owns the fundamental ECS resources and systems used by all
//! front-end layers.  It exposes the [`AutomataRegistry`] and [`RuleRegistry`]
//! as well as the stepping logic that advances every automaton.  Rendering and
//! user-interface crates depend on this core for a stable API.

pub mod prelude;

pub mod automata;
pub mod systems;
pub mod world;

pub mod events;
pub mod plugin;

