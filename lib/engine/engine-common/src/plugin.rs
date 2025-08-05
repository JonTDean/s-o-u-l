//! # Engine-common plug-in layer
//!
//! A single Bevy `Plugin` that bundles **all run-time services** exposed by
//! `engine-common` so that downstream crates can wire everything up with a
//! one-liner:
//!
//! ```rust
//! use engine_common::prelude::EngineCommonPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(EngineCommonPlugin)   // ← this line
//!     .run();
//! ```
//!
//! ## What gets installed
//! | Sub-system | Plug-in | Responsibilities                                     |
//! |------------|---------|------------------------------------------------------|
//! | Cameras    | [`CameraPluginBundle`](crate::controls::camera::CameraPluginBundle) | *Orthographic manager*, *free-cam*, input facade |
//! | Scenes     | [`SceneManagerPlugin`](crate::scenes::SceneManagerPlugin)      | State-driven scene routing, pause handling        |
//!
//! The bundle is intentionally **order-agnostic** – it contains only isolated
//! systems/resources, so you may add it at any stage without breaking other
//! plug-ins.

use bevy::prelude::*;

use crate::{
    controls::camera::CameraPluginBundle,
    scenes::SceneManagerPlugin,
};

/// Umbrella plug-in – installs every `engine-common` runtime feature.
#[derive(Default)]
pub struct EngineCommonPlugin;

impl Plugin for EngineCommonPlugin {
    /// Registers sub-plug-ins.  All sub-systems are thread-safe and run on
    /// Bevy’s default parallel executor, so no additional scheduling is
    /// required here.
    fn build(&self, app: &mut App) {
        app.add_plugins((
            /* World / UI cameras, pan-zoom controller, optional free-cam */
            CameraPluginBundle,
            /* Scene routing, main-menu + options UI, pause overlay      */
            SceneManagerPlugin,
        ));
    }
}

// /* 2 ░ camera stack
        //  * ── orthographic manager   (UI + world layers, metrics, etc.)
        //     * ── perspective free-cam   (spawns on InGame → flies anywhere)
        //  */
        // app.add_plugins(CameraManagerPlugin);