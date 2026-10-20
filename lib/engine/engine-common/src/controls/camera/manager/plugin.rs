//! Low-level camera manager: spawns the UI & world cameras, keeps
//! diagnostics up to date, and installs built-in input helpers.

use bevy::prelude::*;

use crate::controls::camera::{
    freecam::plugin::FreeCamPlugin,
    manager::{
        CameraMetrics, CameraSet, DragState, ZoomInfo, apply_orbit, begin_drag, drag_pan, end_drag,
        gather_orbit_input, key_pan, refresh_zoom_info, spawn_cameras, stage,
        update_camera_metrics, zoom_keyboard, zoom_scroll,
    },
};
use engine_core::{
    automata::channel::{ChannelSet, Channels},
    prelude::AppState,
};

/// Manages the orthographic cameras plus metrics refresh.
pub struct CameraManagerPlugin;

impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        /* register resources read / written by our systems */
        app.init_resource::<ZoomInfo>()
            .init_resource::<DragState>()
            .init_resource::<CameraMetrics>()
            /* create deterministic stages */
            .configure_sets(
                Update,
                (CameraSet::Input, CameraSet::Heavy.after(CameraSet::Input)),
            )
            /* spawners ---------------------------------------------------- */
            .add_systems(Startup, spawn_cameras)
            /* lightweight input (orthographic editor cam) ---------------- */
            .add_systems(
                Update,
                (
                    // pointer: zoom, drag, orbit — only while the pointer is over the stage
                    (zoom_scroll, begin_drag, drag_pan, gather_orbit_input)
                        .run_if(stage::stage_pointer_allowed),
                    // a drag released outside the stage must still end
                    end_drag,
                    // keyboard: zoom and pan — not while a text field has focus
                    (zoom_keyboard, key_pan).run_if(stage::stage_keys_allowed),
                )
                    .in_set(CameraSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            /* heavy work – metrics + orbit application ------------------ */
            .add_systems(
                Update,
                (refresh_zoom_info, update_camera_metrics, apply_orbit)
                    .in_set(CameraSet::Heavy)
                    .run_if(in_state(AppState::InGame)),
            )
            /* lab stage: viewport, activity and framing ------------------ */
            .init_resource::<stage::StageRect>()
            .add_systems(
                Update,
                (stage::frame_selected, stage::apply_stage)
                    .chain()
                    .in_set(ChannelSet::Settle)
                    .run_if(resource_exists::<Channels>.and(stage::stage_active)),
            )
            /* optional perspective free-cam ----------------------------- */
            .add_plugins(FreeCamPlugin);
    }
}
