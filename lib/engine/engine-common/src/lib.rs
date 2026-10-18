//! # engine_common
//!
//! Cross-crate runtime utilities – cameras, scene management, debugging, and
//! other building blocks shared by **every** executable in the S.O.U.L.
//! codebase.
//
//! ## Usage
//! ```no_run
//! use bevy::prelude::*;
//! use engine_core::{plugin::EngineCorePlugin, prelude::AppState};
//! use engine_common::prelude::*;   // grab the goodies
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .init_state::<AppState>()
//!     .add_plugins(EngineCorePlugin)
//!     .add_plugins(EngineCommonPlugin)   // installs cameras + scenes
//!     .run();
//! ```

#![warn(missing_docs)]

/* -------------------------------------------------------------------- */
/* Public modules                                                       */
/* -------------------------------------------------------------------- */

pub mod controls;
pub mod plugin;
pub mod prelude;
pub mod scenes;

/* -------------------------------------------------------------------- */
/* Re-exports at crate root for quick access                            */
/* -------------------------------------------------------------------- */

pub use plugin::EngineCommonPlugin;
