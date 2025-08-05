//! Renderer **material assets** and their helper plug-ins.
//!
//! Everything that ends up in a GPU shader lives under this module.  
//!
//! * `automata` – 2-D material drawing a full cellular-automata grid.  
//! * `debug`    – tiny helper materials such as the checker-board floor.  
//! * `plugin`   – bundles all sub-plug-ins so the main renderer can just  
//!                call `MaterialsPlugin` once.
#![allow(clippy::module_name_repetitions)]   // optional quality-of-life

/// GPU material that draws a full automata grid (see [`automata`]).
pub mod automata;

/// Visual debugging helpers such as the checkerboard background.
pub mod debug;

/// Umbrella plug-in that registers every material sub-plug-in.
pub mod plugin;