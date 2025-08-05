//! Orthographic *world-camera* controller (panning / zoom / follow).
//!
//! All mutable state lives inside [`CameraController`] so concurrent
//! systems can **read** it safely – eg. UI widgets, minimap, debug gizmos.
//!
//! ## Highlights
//! * **Modes** – `Free`, `Follow`, `Recenter`.
//! * **Predictable zoom** – physical world-units per screen-pixel.
//! * **Pixel-perfect pan** – keyboard + mouse with zoom compensation.
//! * **Thread-friendly** – input collection in `CameraSet::Input`,
//!   heavy math in `CameraSet::Heavy`.

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::Projection,
};
use engine_core::prelude::{AutomataRegistry, AutomatonId};

use super::{
    systems::{
        fit_or_clamp_camera, DragState, WorldCamera, KEY_PAN_SPEED, MAX_SCALE, MIN_SCALE_CONST,
        ZOOM_FACTOR,
    },
    CameraSet,
};

/* =================================================================== */
/* CameraController resource                                           */
/* =================================================================== */

/// Current navigation mode.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Unconstrained panning / zooming.
    Free,
    /// Keep the AABB of *all* automatons visible (classic “follow”).
    Follow,
    /// One-shot recenter to show *all* automatons once.
    Recenter,
}

/// Global resource: the *single source of truth* for camera state.
#[derive(Resource, Debug)]
pub struct CameraController {
    /// Current mode.
    pub mode: Mode,
    /// Automaton currently tracked (optional; may be unused for now).
    pub target: Option<AutomatonId>,
    /// Physical world-units per screen-pixel (orthographic *scale*).
    pub zoom: f32,
    /// Configurable limits.
    pub min_zoom: f32,
    pub max_zoom: f32,
    /// Pan motion accumulated this frame (world units).
    pub pan_delta: Vec2,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            mode: Mode::Free,
            target: None,
            zoom: 1.0,
            min_zoom: MIN_SCALE_CONST,
            max_zoom: MAX_SCALE,
            pan_delta: Vec2::ZERO,
        }
    }
}

/* =================================================================== */
/* Plugin                                                              */
/* =================================================================== */

/// Bundles the resource + tick systems.
pub struct CameraControllerPlugin;
impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ resources */
        app.init_resource::<CameraController>()
           .init_resource::<DragState>();

        /* 2 ░ systems – input then heavy update */
        app.add_systems(
            Update,
            (
                gather_input.in_set(CameraSet::Input),
                apply_camera_controller.in_set(CameraSet::Heavy),
            ),
        );
    }
}

/* =================================================================== */
/* System: gather_input (light-weight)                                 */
/* =================================================================== */

/// Polls input devices and mutates [`CameraController`].
///
/// * **Mouse wheel** → zoom  
/// * **WASD / arrows** → pan  
/// * **RMB / MMB drag** → pan under cursor  
/// * **C** key → recenter once  
/// * **F** key → toggle follow
#[allow(clippy::too_many_arguments)]
fn gather_input(
    mut ctrl:   ResMut<CameraController>,
    mut drag:   ResMut<DragState>,
    mut wheel:  EventReader<MouseWheel>,
    keys:       Res<ButtonInput<KeyCode>>,
    buttons:    Res<ButtonInput<MouseButton>>,
    windows:    Query<&Window>,
    time:       Res<Time>,
) {
    /* ── zoom (scroll wheel) ───────────────────────────────────────── */
    let scroll: f32 = wheel.read().map(|e| e.y).sum();
    if scroll != 0.0 {
        let factor = if scroll > 0.0 { 1.0 / ZOOM_FACTOR } else { ZOOM_FACTOR };
        ctrl.zoom = (ctrl.zoom * factor).clamp(ctrl.min_zoom, ctrl.max_zoom);
    }

    /* ── keyboard pan (WASD / arrows) ─────────────────────────────── */
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }

    if dir != Vec2::ZERO {
        let delta = dir.normalize()
            * KEY_PAN_SPEED
            * ctrl.zoom
            * time.delta_secs();
        ctrl.pan_delta += delta;
    }

    /* ── mouse drag pan (RMB / MMB) ───────────────────────────────── */
    let pressed = buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle);

    if pressed {
        // start drag if not already
        if drag.0.is_none() {
            drag.0 = windows.single().ok().and_then(|w| w.cursor_position());
        }

        if let (Some(prev), Ok(win)) = (drag.0, windows.single()) {
            if let Some(cur) = win.cursor_position() {
                let win_size  = Vec2::new(win.width(), win.height());
                let ndc_prev  = (prev / win_size) * 2.0 - Vec2::ONE;
                let ndc_cur   = (cur  / win_size) * 2.0 - Vec2::ONE;

                let delta = (ndc_cur - ndc_prev) * win_size * 0.5 * ctrl.zoom;
                ctrl.pan_delta += Vec2::new(-delta.x,  delta.y);
                drag.0 = Some(cur);
            }
        }
    } else {
        drag.0 = None;
    }

    /* ── mode toggles ──────────────────────────────────────────────── */
    if keys.just_pressed(KeyCode::KeyC) {
        ctrl.mode = Mode::Recenter;
    }
    if keys.just_pressed(KeyCode::KeyF) {
        ctrl.mode = if matches!(ctrl.mode, Mode::Follow) { Mode::Free } else { Mode::Follow };
    }
}

/* =================================================================== */
/* System: apply_camera_controller (heavy)                             */
/* =================================================================== */
#[allow(clippy::too_many_arguments)]
fn apply_camera_controller(
    mut ctrl:      ResMut<CameraController>,
    registry:      Res<AutomataRegistry>,
    windows:       Query<&Window>,
    mut cam_q:     Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    let (Ok(win), Ok((mut tf, mut proj))) = (windows.single(), cam_q.single_mut()) else { return };

    /* ── apply pan & zoom first ───────────────────────────────────── */
    tf.translation.x += ctrl.pan_delta.x;
    tf.translation.y += ctrl.pan_delta.y;
    ctrl.pan_delta = Vec2::ZERO; // reset accumulator

    if let Projection::Orthographic(ref mut o) = *proj {
        o.scale = ctrl.zoom;
    }

    /* ── mode-specific logic ──────────────────────────────────────── */
    match ctrl.mode {
        Mode::Free => {}
        Mode::Recenter | Mode::Follow => {
            if registry.list().is_empty() {
                ctrl.mode = Mode::Free;
                return;
            }

            /* aggregate AABB from all automatons */
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);

            for info in registry.list() {
                let off  = info.world_offset;
                let size = Vec3::new(
                    info.slice.size.x as f32,
                    info.slice.size.y as f32,
                    1.0,
                ) * info.voxel_size;

                min = min.min(off);
                max = max.max(off + size);
            }

            if let Projection::Orthographic(ref mut o) = *proj {
                let (centre, scale) =
                    fit_or_clamp_camera(min, max, win, tf.translation, o.scale);
                tf.translation.x = centre.x;
                tf.translation.y = centre.y;
                o.scale          = scale;
                ctrl.zoom        = scale;
            }

            if matches!(ctrl.mode, Mode::Recenter) {
                ctrl.mode = Mode::Free;
            }
        }
    }

    /* floating-origin compatible: Z stays high so we never cross UI */
    tf.translation.z = 1_000.0;
}
