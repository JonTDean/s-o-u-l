//! Public facade of the **render** subsystem.
pub mod camera;
pub mod materials;
pub mod minimap;
pub mod interpolator;

pub use camera::{CameraPlugin, WorldCamera, ZoomInfo};
