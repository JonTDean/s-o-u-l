//! Unified camera stack – controller, systems, free-cam and low-level input.
//!
//! This entire implementation lives in **engine-common**, so any crate
//! (*engine-render*, editors, test harnesses, …) can obtain consistent
//! camera behaviour by adding a single plugin bundle:
//!
//! ```rust
//! use engine_common::controls::camera::CameraPluginBundle;
//!
//! App::new()
//!     .add_plugins(CameraPluginBundle)   // ← one-liner
//!     .run();
//! ```

/* ──────────────────────── sub-modules ─────────────────────────────── */
pub mod systems;
pub mod controller;
pub mod freecam;
pub mod input;

/* ─────────────────── convenient re-exports ────────────────────────── */
pub use controller::CameraControllerPlugin;
pub use freecam::FreeCamPlugin;
pub use systems::{
    CameraMetrics, CameraSet, DragState, ViewportRect, WorldCamera, ZoomInfo,
    KEY_PAN_SPEED, MAX_SCALE, MIN_SCALE_CONST, ZOOM_FACTOR,
};

/* -------------------------------------------------------------------- */
/* Umbrella plugin – installs everything in the right order.            */
/* -------------------------------------------------------------------- */

use bevy::prelude::*;

/// Add *all* camera-related plugins & systems in one call.
///
/// * **CameraManagerPlugin** – low-level orthographic cameras + metrics  
/// * **CameraControllerPlugin** – 2-D world-camera (pan/zoom/follow)  
/// * **FreeCamPlugin** – optional perspective free-camera (inactive until
///   you spawn it with `spawn_freecam`)  
/// * **InputFacadePlugin** – lightweight input helpers used by the UI
pub struct CameraPluginBundle;

impl Plugin for CameraPluginBundle {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                /* low-level glue, floating-origin hooks, etc. */
                systems::CameraManagerPlugin,
                /* primary orthographic controller */
                controller::CameraControllerPlugin,
                /* optional perspective free-cam */
                freecam::FreeCamPlugin,
            ))
            /* lightweight input facade (zoom, drag, pan, orbit) */
            .add_plugins(input::InputFacadePlugin)
            /* make sure shared resources are present */
            .init_resource::<ZoomInfo>()
            .init_resource::<DragState>()
            .init_resource::<systems::CameraMetrics>()
            .init_resource::<input::rotate::OrbitAngles>()
            /* spawn the UI + world cameras right away so the main menu,
               splash screen, etc. can render before `AppState::InGame` */
            .add_systems(Startup, systems::spawn_cameras);
    }
}
