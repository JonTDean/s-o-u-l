//! Convenient re-exports for downstream crates.
//!
//! ```rust
//! use engine_common::prelude::*;   // grab everything
//! ```
//!
//! * **EngineCommonPlugin** – the all-in-one plug-in layer.
//! * Camera helpers – spawn functions, constants, components.
//! * Scene helpers – pause actions, state plug-ins.
//
//! Importing the prelude never pulls heavy dependencies such as *egui* into
//! the public API; they stay behind the scenes.

pub use crate::plugin::EngineCommonPlugin;

/* ── cameras ───────────────────────────────────────────────────────── */
pub use crate::controls::camera::{
    spawn_cameras,           /* helper for tests / custom bootstrap   */
    freecam::spawn_freecam,  /* convenience for spawning the free-cam */
    CameraMetrics,
    CameraSet,
    KEY_PAN_SPEED,
};

/* ── scenes & pause routing ───────────────────────────────────────── */
pub use crate::scenes::{
    PauseAction,
    SceneManagerPlugin,
};
