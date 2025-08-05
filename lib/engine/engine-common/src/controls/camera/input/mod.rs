//! Mouse / keyboard *facade* – re-exports concrete helpers so external
//! crates can simply `use camera::input::*` without deep paths.

pub mod zoom;
pub mod drag;
pub mod pan;
pub mod rotate;
pub mod plugin; 

/* public re-exports ------------------------------------------------- */
pub use zoom::{zoom_scroll, zoom_keyboard};
pub use drag::{begin_drag, drag_pan, end_drag};
pub use pan::key_pan;
pub use rotate::{gather_orbit_input, apply_orbit, OrbitAngles};
pub use plugin::InputFacadePlugin;
