//! Low-level camera manager: spawns the UI & world cameras, keeps
//! diagnostics up to date, and installs built-in input helpers.

use bevy::prelude::*;

use crate::controls::camera::{
    manager::{
        CameraMetrics, CameraSet, DragState, ZoomInfo,
        apply_orbit, begin_drag, drag_pan, end_drag, gather_orbit_input,
        key_pan, refresh_zoom_info, spawn_cameras, update_camera_metrics,
        zoom_keyboard, zoom_scroll,
    },
    freecam::plugin::FreeCamPlugin,
};
use engine_core::prelude::AppState;

/// Manages the orthographic cameras plus metrics refresh.
pub struct CameraManagerPlugin;

impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        /* register resources read / written by our systems */
        app.init_resource::<ZoomInfo>()
           .init_resource::<DragState>()
           .init_resource::<CameraMetrics>()

           /* create deterministic stages */
           .configure_sets(Update, (CameraSet::Input, CameraSet::Heavy.after(CameraSet::Input)))

           /* spawners ---------------------------------------------------- */
           .add_systems(Startup, spawn_cameras)

           /* lightweight input (orthographic editor cam) ---------------- */
           .add_systems(
               Update,
               (
                   // zoom
                   zoom_scroll, zoom_keyboard,
                   // drag
                   begin_drag, drag_pan, end_drag,
                   // pan
                   key_pan,
                   // orbit (optional)
                   gather_orbit_input,
               )
               .in_set(CameraSet::Input)
               .run_if(in_state(AppState::InGame)),
           )

           /* heavy work – metrics + orbit application ------------------ */
           .add_systems(
               Update,
               (
                   refresh_zoom_info,
                   update_camera_metrics,
                   apply_orbit,
               )
               .in_set(CameraSet::Heavy)
               .run_if(in_state(AppState::InGame)),
           )

           /* optional perspective free-cam ----------------------------- */
           .add_plugins(FreeCamPlugin);
    }
}
