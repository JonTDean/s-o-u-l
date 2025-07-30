//! Public face of the **render** subsystem.

pub mod camera;
pub mod active;
pub mod worldgrid;
pub mod material;

pub use camera::{CameraPlugin, WorldCamera, ZoomInfo, WORLD_LAYER, UI_LAYER};
pub use active::plugin::AutomataRenderMap;
pub use worldgrid::WorldGrid;
pub use material::{AutomataMaterial, AutomataParams};
