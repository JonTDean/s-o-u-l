//! Collection of **run-time visual debugging aids**.
//!
//! At the moment two helper materials are exposed:
//!
//! | Module               | Purpose                                                         |
//! |----------------------|-----------------------------------------------------------------|
//! | `debug_floor`        | Pixel-perfect floor grid rendered with [`Material2d`] – useful  |
//! |                      | for gauging zoom and camera panning accuracy.                   |
//! | `debug_grid`         | High-contrast checkerboard shown *behind* the world to aid      |
//! |                      | orientation in large 2-D scenes.                                |
//!
//! The top-level [`plugin`] sub-module re-exports a tiny Bevy plug-in that bundles these
//! materials so downstream crates can simply call<br/>
//! `app.add_plugins(DebugMaterialsPlugin);`

/// Low-level material that actually draws the grid lines.
pub mod debug_floor;

/// Tiny plug-in that spawns + drives a zoom-aware grid entity.
pub mod debug_grid;

/// Bundles the two helpers above into a single Bevy plug-in.
pub mod plugin;
