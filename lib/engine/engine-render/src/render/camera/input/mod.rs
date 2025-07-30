//! Mouse / keyboard input facade – re-exports the concrete helpers so
//! external code can keep calling `camera::input::*`.

mod zoom;
mod drag;
mod pan;

pub use zoom::{zoom_scroll, zoom_keyboard};
pub use drag::{begin_drag, drag_pan, end_drag};
pub use pan::key_pan;
