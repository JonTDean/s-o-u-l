//! Public facade of the **render** subsystem.
pub mod camera;
pub mod material;
pub mod minimap;
pub mod interpolator;

pub use camera::{CameraPlugin, WorldCamera, ZoomInfo};
pub use material::{AutomataMaterial, AutomataParams};
