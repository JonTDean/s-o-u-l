//! Unified camera stack – shared data-types & helpers.
//!
//! *Every* concrete `Plugin` now lives in its own file so that this
//! façade stays lightweight and purely declarative.

//! Unified camera stack – shared data-types & helpers.

pub mod manager;
pub mod controller;
pub mod freecam;
pub mod input;

/* plug-ins ---------------------------------------------------------- */
pub mod plugin_bundle;                                  // umbrella
pub use plugin_bundle::CameraPluginBundle;
pub use controller::plugin::CameraControllerPlugin;
pub use freecam::plugin::FreeCamPlugin;
pub use input::plugin::InputFacadePlugin;
pub use manager::plugin::CameraManagerPlugin;

/* shared re-exports ------------------------------------------------ */
pub use manager::{
    CameraMetrics, CameraSet, DragState, ViewportRect, WorldCamera, ZoomInfo,
    KEY_PAN_SPEED, MAX_SCALE, MIN_SCALE_CONST, ZOOM_FACTOR, spawn_cameras, // <-- added
};
pub use controller::{CameraController, Mode};
pub use freecam::{FreeCamSettings, spawn_freecam};
