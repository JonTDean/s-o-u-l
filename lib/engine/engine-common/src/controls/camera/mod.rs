//! Unified camera stack – shared data-types & helpers.
//!
//! *Every* concrete `Plugin` now lives in its own file so that this
//! façade stays lightweight and purely declarative.

//! Unified camera stack – shared data-types & helpers.

pub mod manager;
pub mod motion;
pub use motion::{controller, freecam};

pub mod input;

/* plug-ins ---------------------------------------------------------- */
pub mod plugin_bundle; // umbrella
pub use controller::plugin::CameraControllerPlugin;
pub use freecam::plugin::FreeCamPlugin;
pub use input::plugin::InputFacadePlugin;
pub use manager::plugin::CameraManagerPlugin;
pub use plugin_bundle::CameraPluginBundle;

/* shared re-exports ------------------------------------------------ */
pub use controller::{CameraController, Mode};
pub use freecam::{FreeCamSettings, spawn_freecam};
pub use manager::{
    CameraMetrics,
    CameraSet,
    DragState,
    KEY_PAN_SPEED,
    MAX_SCALE,
    MIN_SCALE_CONST,
    ViewportRect,
    WorldCamera,
    ZOOM_FACTOR,
    ZoomInfo,
    spawn_cameras, // <-- added
};
