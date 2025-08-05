//! # engine_common
//!
//! Cross-crate runtime utilities – cameras, scene management, debugging, and
//! other building blocks shared by **every** executable in the S.O.U.L.
//! codebase.
//
//! ## Usage
//! ```rust
//! use engine_common::prelude::*;   // grab the goodies
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(EngineCommonPlugin)   // installs cameras + scenes
//!     .run();
//! ```

#![warn(missing_docs)]

/* -------------------------------------------------------------------- */
/* Public modules                                                       */
/* -------------------------------------------------------------------- */

pub mod controls;
pub mod scenes;
pub mod plugin;
pub mod prelude; 

/* -------------------------------------------------------------------- */
/* Re-exports at crate root for quick access                            */
/* -------------------------------------------------------------------- */

pub use plugin::EngineCommonPlugin;
