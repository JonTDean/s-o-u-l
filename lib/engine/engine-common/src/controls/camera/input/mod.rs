//! Mouse / keyboard *facade* – re-exports concrete helpers so external
//! crates can simply `use camera::input::*` without diving into sub-modules.

pub mod zoom;
pub mod drag;
pub mod pan;
pub mod rotate;

/* ── public re-exports ───────────────────────────────────────────── */
pub use zoom::{zoom_scroll, zoom_keyboard};
pub use drag::{begin_drag, drag_pan, end_drag};
pub use pan::key_pan;
pub use rotate::{gather_orbit_input, apply_orbit, OrbitAngles};

/* ------------------------------------------------------------------ */
/* Plugin: installs input helpers into the correct `CameraSet`.       */
/* ------------------------------------------------------------------ */

use bevy::prelude::*;
use crate::controls::camera::{CameraSet, systems};
use engine_core::prelude::AppState;

pub struct InputFacadePlugin;
impl Plugin for InputFacadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                zoom_scroll,
                zoom_keyboard,
                begin_drag,
                drag_pan,
                end_drag,
                key_pan,
                gather_orbit_input,
            )
            .in_set(CameraSet::Input)
            .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            apply_orbit
                .in_set(CameraSet::Heavy)
                .run_if(in_state(AppState::InGame)),
        )
        // ensure DragState exists even if the main controller plugin
        // is disabled (editor-only usage, headless tests…)
        .init_resource::<systems::DragState>();
    }
}
