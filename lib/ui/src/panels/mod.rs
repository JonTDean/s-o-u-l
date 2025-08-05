//! Top-level module for UI menus.
//!
//! * Removes all traces of the deprecated **`UiCameraConfig`** component.
//! * Spawns a dedicated *UI camera* using the modern `Camera2d` component
//!   (all required components are inserted automatically).
//!
//! The UI camera renders **after** the world camera by giving it a larger
//! `Camera::order` value.


pub mod world;
pub mod plugin;