//! always points from the active camera to the *currently selected* automaton.
//!
//! ## Overview
//! * A single Bevy system (`draw_selection_arrow`) runs every frame **in-game**.
//! * It queries the [`MinimapSelection`] resource for the chosen automaton’s
//!   `AutomatonId` and looks up its position via [`AutomataRegistry`].
//! * If both the camera and the automaton exist, it draws a red arrow gizmo:
//!   * **shaft** – straight line camera → centre-of-automaton.
//!   * **head**  – two short lines forming a 30 ° V-shape.
//! * Arrow length and head size adapt automatically; the system is thread-safe
//!   (read-only queries + exclusive `&mut Gizmos`).
//!
//! ## Multi-threading
//! The system only *reads* from world resources/components (`Query<&GlobalTransform>`
//! and `Res<_>`), so Bevy can schedule it in parallel with any other read-only
//! systems.  `Gizmos` is schedule-exclusive by design, guaranteeing no renderer
//! contention.
//!
//! ## Usage
//! Add `SelectionArrowPlugin` to your `App` **after** the camera and automata
//! plugins are registered:
//! ```no_run
//! use ui::panels::world::automata::selection_arrow::SelectionArrowPlugin;
//! # use bevy::prelude::*;
//! # fn main() {
//!     App::new()
//!         // … engine_core, engine_render, ui, etc …
//!         .add_plugins(SelectionArrowPlugin)
//!         .run();
//! # }
//! ```

use bevy::prelude::*;
use engine_core::prelude::{AppState, AutomataRegistry};
use engine_render::WorldCamera;
use simulation_kernel::grid::GridBackend;

// UI layer – selected automaton ID comes from the minimap overlay.
use crate::panels::world::minimap_overlay::MinimapSelection;

/* --------------------------------------------------------------------- */
/*                              Plugin                                   */
/* --------------------------------------------------------------------- */

/// Single-system plugin that renders the selection arrow every frame.
pub struct SelectionArrowPlugin;
impl Plugin for SelectionArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            draw_selection_arrow
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/* --------------------------------------------------------------------- */
/*                             Constants                                 */
/* --------------------------------------------------------------------- */

/// Colour of the selection arrow (SRGB – bright red).
const ARROW_COLOUR: Color = Color::srgb(1.0, 0.0, 0.0);
/// Length of each arrow-head leg as a fraction of the shaft length
/// (clamped by `ARROW_HEAD_MAX`).
const ARROW_HEAD_FRAC: f32 = 0.1;
/// Hard upper limit so the head does not grow unbounded for very long shafts.
const ARROW_HEAD_MAX:  f32 = 50.0;
/// Angle (deg) between the shaft and each head leg.
const ARROW_HEAD_ANGLE: f32 = 30.0_f32.to_radians();

/* --------------------------------------------------------------------- */
/*                              System                                   */
/* --------------------------------------------------------------------- */

/// Draw a red arrow from the camera to the currently-selected automaton.
#[allow(clippy::too_many_arguments)]
fn draw_selection_arrow(
    sel:       Res<MinimapSelection>,
    registry:  Res<AutomataRegistry>,
    cam_q:     Query<&GlobalTransform, With<WorldCamera>>,  // read-only
    mut gizmos: Gizmos,                                     // exclusive writer
) {
    // Auto-clear: Bevy completely refreshes gizmos every frame – no manual
    // clearing required here.

    let Some(sel_id) = sel.0 else { return }; // no selection ⇒ nothing to draw
    let Some(info)   = registry.get(sel_id) else { return };
    let Ok(cam_xf)   = cam_q.single()           else { return };

    // 1 ░ world-space positions
    let cam_pos   = cam_xf.translation();

    let grid_sz   = match &info.grid {
        GridBackend::Dense(g)  => Vec2::new(g.size.x as f32, g.size.y as f32),
        GridBackend::Sparse(_) => Vec2::splat(512.0),
    } * info.cell_size;

    let auto_centre = info.world_offset + grid_sz * 0.5;
    let auto_pos    = Vec3::new(auto_centre.x, auto_centre.y, cam_pos.z); // keep Z

    // 2 ░ shaft (camera → automaton)
    gizmos.line(cam_pos, auto_pos, ARROW_COLOUR);

    // 3 ░ arrow-head – two legs forming a 30deg V-shape
    //     NOTE: use (auto - cam) so the head points *toward* the automaton.
    let dir   = (auto_pos - cam_pos).truncate();
    let len   = dir.length();
    if len == 0.0 { return; }
    let dir_n = dir / len; // normalised 2-D direction

    let head_len = (len * ARROW_HEAD_FRAC).min(ARROW_HEAD_MAX);

    // Rotate dir by ±ANGLE to get head legs
    let (sin_a, cos_a) = ARROW_HEAD_ANGLE.sin_cos();
    let rot_left  = Vec2::new(
        dir_n.x * cos_a - dir_n.y * sin_a,
        dir_n.x * sin_a + dir_n.y * cos_a,
    );
    let rot_right = Vec2::new(
        dir_n.x * cos_a + dir_n.y * sin_a,
        -dir_n.x * sin_a + dir_n.y * cos_a,
    );

    let left_end  = auto_pos + rot_left .extend(0.0) * head_len;
    let right_end = auto_pos + rot_right.extend(0.0) * head_len;

    gizmos.line(auto_pos, left_end,  ARROW_COLOUR);
    gizmos.line(auto_pos, right_end, ARROW_COLOUR);
}
