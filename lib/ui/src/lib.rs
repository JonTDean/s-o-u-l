//! User interface layer built with `egui`.
//!
//! This crate contains all menu screens and heads-up displays shown during
//! gameplay.  It re-exports [`OutputPlugin`], which bundles every UI panel and
//! the CPU-based renderer for active automata grids.

pub mod panels;
pub mod styles;
pub mod plugin;