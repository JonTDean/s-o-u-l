//! Mouse / keyboard *facade* – re-exports concrete helpers so external
//! crates can simply `use camera::input::*` without deep paths.

pub mod translation;
pub mod zoom;
pub use translation::{drag, pan};

pub mod plugin;
pub mod rotate;

/* public re-exports ------------------------------------------------- */
pub use drag::{begin_drag, drag_pan, end_drag};
pub use pan::key_pan;
pub use plugin::InputFacadePlugin;
pub use rotate::{OrbitAngles, apply_orbit, gather_orbit_input};
pub use zoom::{zoom_keyboard, zoom_scroll};
