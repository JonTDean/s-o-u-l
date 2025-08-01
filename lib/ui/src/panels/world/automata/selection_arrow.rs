//! always points from the active camera to the *currently selected* automaton.
//! (updated 2025-08-01 – uses `AutomatonInfo::slice`)

use bevy::prelude::*;
use engine_core::prelude::{AppState, AutomataRegistry};
use engine_render::WorldCamera;

use crate::panels::world::minimap_overlay::MinimapSelection;

/* --------------------------------------------------------------------- */
/*                              Plugin                                   */
/* --------------------------------------------------------------------- */
pub struct SelectionArrowPlugin;
impl Plugin for SelectionArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_selection_arrow.run_if(in_state(AppState::InGame)));
    }
}

/* --------------------------------------------------------------------- */
/*                             Constants                                 */
/* --------------------------------------------------------------------- */
const ARROW_COLOUR:       Color = Color::srgb(1.0, 0.0, 0.0);
const ARROW_HEAD_FRAC:    f32   = 0.1;
const ARROW_HEAD_MAX:     f32   = 50.0;
const ARROW_HEAD_ANGLE:   f32   = 30.0_f32.to_radians();

/* --------------------------------------------------------------------- */
/*                              System                                   */
/* --------------------------------------------------------------------- */
fn draw_selection_arrow(
    sel:       Res<MinimapSelection>,
    registry:  Res<AutomataRegistry>,
    cam_q:     Query<&GlobalTransform, With<WorldCamera>>,
    mut gizmos: Gizmos,
) {
    let Some(sel_id) = sel.0                else { return };
    let Some(info)   = registry.get(sel_id) else { return };
    let Ok(cam_xf)   = cam_q.single()       else { return };

    /* 1 ─ positions ---------------------------------------------------- */
    let cam_pos   = cam_xf.translation();
    let grid_sz_xy =
        Vec2::new(info.slice.size.x as f32, info.slice.size.y as f32) * info.voxel_size;
    let auto_centre = info.world_offset + grid_sz_xy.extend(0.0) * 0.5;
    let auto_pos    = Vec3::new(auto_centre.x, auto_centre.y, cam_pos.z); // lock Z plane

    /* 2 ─ shaft -------------------------------------------------------- */
    gizmos.line(cam_pos, auto_pos, ARROW_COLOUR);

    /* 3 ─ arrow head --------------------------------------------------- */
    let dir   = (auto_pos - cam_pos).truncate();
    let len   = dir.length();
    if len == 0.0 { return; }
    let dir_n = dir / len;

    let head_len = (len * ARROW_HEAD_FRAC).min(ARROW_HEAD_MAX);
    let (sin_a, cos_a) = ARROW_HEAD_ANGLE.sin_cos();

    let rot_left  = Vec2::new(dir_n.x *  cos_a - dir_n.y * sin_a,
                              dir_n.x *  sin_a + dir_n.y * cos_a);
    let rot_right = Vec2::new(dir_n.x *  cos_a + dir_n.y * sin_a,
                             -dir_n.x *  sin_a + dir_n.y * cos_a);

    gizmos.line(auto_pos, auto_pos + rot_left .extend(0.0) * head_len, ARROW_COLOUR);
    gizmos.line(auto_pos, auto_pos + rot_right.extend(0.0) * head_len, ARROW_COLOUR);
}
