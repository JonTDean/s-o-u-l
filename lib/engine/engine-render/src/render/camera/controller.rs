//! render/camera/controller.rs ─ unified *world‑camera* control layer
//! ------------------------------------------------------------------
//! This module supersedes the legacy `camera::systems` stack by
//! concentrating **all** mutable camera state in one [`CameraController`]
//! resource and driving the [`WorldCamera`] transform/zoom from a single
//! system.  The design follows the v1.0 camera‑rework spec (2025‑07‑30).
//!
//! ### Highlights
//! * **Modes** – *Free*, *Follow(id)*, *Recenter*.
//! * **Predictable zoom** – scroll wheel sets *physical world‑units per
//!   screen‑pixel* (`zoom`); `min_zoom`/`max_zoom` clamped.
//! * **Pixel‑perfect panning** – WASD / arrow / RMB‑drag accumulates a
//!   `pan_delta` which is applied in world‑space each frame.
//! * **Thread‑safe & parallel‑friendly** – input collection runs in the
//!   light‑weight `MainSet::Input`; the heavy camera update runs in
//!   `MainSet::Logic` so floating‑origin & gizmos can follow.
//!
//! ### Public surface
//! ```rust
//! pub struct CameraControllerPlugin;      // add to your App()
//! pub struct CameraController;            // resource (read‑only for UI)
//! ```
//!
//! ### Scheduling
//! ```text
//! ┌── MainSet::Input ─────────────────────────────────────────┐
//! │ gather_input → mut CameraController                      │
//! └───────────────────────────────────────────────────────────┘
//! ┌── MainSet::Logic ─────────────────────────────┐
//! │ apply_camera_controller → mut Transform       │
//! │                               mut Projection  │
//! └───────────────────────────────────────────────┘
//! ```
//! The plugin automatically inserts itself into these sets.
//!
//! ------------------------------------------------------------------

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::Projection,
};
use engine_core::prelude::{AutomataRegistry, AutomatonId};
use simulation_kernel::grid::GridBackend;
use crate::render::camera::systems::{fit_or_clamp_camera, WorldCamera, KEY_PAN_SPEED, MAX_SCALE, MIN_SCALE_CONST, ZOOM_FACTOR};
use crate::render::camera::floating_origin::WorldOffset;
use crate::render::camera::systems::spawn_cameras; // reuse helper
use crate::render::camera::systems::DragState;

/* ===================================================================== */
/* CameraController resource                                             */
/* ===================================================================== */

/// Operating mode for the world camera.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Unconstrained panning / zooming.
    Free,
    /// Keep the AABB of *all* automatons visible (classic ‘follow’).
    Follow,
    /// One‑shot recenter to show *all* automatons once.
    Recenter,
}

/// Global resource: the *single source of truth* for mutable camera state.
#[derive(Resource, Debug)]
pub struct CameraController {
    /// Current navigation mode.
    pub mode: Mode,
    /// ID of the automaton that was most recently clicked in the HUD / minimap.
    pub target: Option<AutomatonId>,
    /// Physical world‑units per screen‑pixel (orthographic *scale*).
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

/* ===================================================================== */
/* Plugin                                                                */
/* ===================================================================== */

/// Bundle the controller resource + systems.
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        use engine_core::systems::schedule::MainSet;

        // 1 ░ ensure world/ UI cameras exist (reuse helper)
        app.add_systems(Startup, spawn_cameras);

        // 2 ░ resources
        app.init_resource::<CameraController>()
           .init_resource::<DragState>();

        // 3 ░ systems: input collection → heavy camera update
        app.add_systems(
            Update,
            (
                gather_input.in_set(MainSet::Input),
                apply_camera_controller.in_set(MainSet::Logic),
            ),
        );
    }
}

/* ===================================================================== */
/* System: gather_input (light‑weight)                                    */
/* ===================================================================== */

/// Polls all relevant input devices and mutates [`CameraController`].
///
/// *Mouse wheel* → zoom.
/// *WASD / arrows* → pan.
/// *RMB / MMB drag* → pan under cursor.
/// *C* key        → one‑shot recenter.
/// *F* key        → toggle follow.
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
    /* ── zoom (scroll wheel) ─────────────────────────────────────── */
    let scroll: f32 = wheel.read().map(|e| e.y).sum();
    if scroll != 0.0 {
        let factor = if scroll > 0.0 { 1.0 / ZOOM_FACTOR } else { ZOOM_FACTOR };
        ctrl.zoom = (ctrl.zoom * factor).clamp(ctrl.min_zoom, ctrl.max_zoom);
    }

    /* ── keyboard pan (WASD / arrows) ────────────────────────────── */
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }

    if dir != Vec2::ZERO {
        // convert *screen* dir into *world* delta using current zoom
        let delta = dir.normalize()
            * KEY_PAN_SPEED
            * ctrl.zoom
            * time.delta_secs();
        ctrl.pan_delta += delta;
    }

    /* ── mouse drag pan (RMB / MMB) ──────────────────────────────── */
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
                // translate by *world* units, note Y inversion (screen ↔ world)
                let delta = (ndc_cur - ndc_prev) * win_size * 0.5 * ctrl.zoom;
                ctrl.pan_delta += Vec2::new(-delta.x,  delta.y);
                drag.0 = Some(cur);
            }
        }
    } else {
        drag.0 = None;
    }

    /* ── mode toggles ─────────────────────────────────────────────── */
    if keys.just_pressed(KeyCode::KeyC) {
        ctrl.mode = Mode::Recenter;
    }
    if keys.just_pressed(KeyCode::KeyF) {
        ctrl.mode = if matches!(ctrl.mode, Mode::Follow) { Mode::Free } else { Mode::Follow };
    }
}

/* ===================================================================== */
/* System: apply_camera_controller (heavy)                                */
/* ===================================================================== */

/// Applies the accumulated controller state to the [`WorldCamera`] each frame.
#[allow(clippy::too_many_arguments)]
fn apply_camera_controller(
    mut ctrl:      ResMut<CameraController>,
    registry:      Res<AutomataRegistry>,
    windows:       Query<&Window>,
    mut cam_q:     Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
    mut offset:    ResMut<WorldOffset>,
) {
    let (Ok(win), Ok((mut tf, mut proj))) = (windows.single(), cam_q.single_mut()) else { return };

    /* ── zoom — apply immediately ─────────────────────────────────── */
    if let Projection::Orthographic(ref mut ortho) = *proj {
        ortho.scale = ctrl.zoom;
    }

    /* ── pan — accumulate then reset in resource ──────────────────── */
    tf.translation.x += ctrl.pan_delta.x;
    tf.translation.y += ctrl.pan_delta.y;
    ctrl.pan_delta = Vec2::ZERO;

    /* ── mode‑specific corrections ────────────────────────────────── */
    match ctrl.mode {
        Mode::Free => {}
        Mode::Recenter | Mode::Follow => {
            if registry.list().is_empty() {
                ctrl.mode = Mode::Free;
                return;
            }

            // Compute world AABB over *all* automatons (future: use target).
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for info in registry.list() {
                let off  = info.world_offset;
                let size = match &info.grid {
                    GridBackend::Dense(g)  => Vec2::new(g.size.x as f32, g.size.y as f32) * info.cell_size,
                    GridBackend::Sparse(_) => Vec2::splat(512.0) * info.cell_size,
                };
                min = min.min(off);
                max = max.max(off + size);
            }

            if let Projection::Orthographic(ref mut o) = *proj {
                let (centre, scale) =
                    fit_or_clamp_camera(min, max, win, tf.translation.truncate(), o.scale);
                tf.translation.x = centre.x;
                tf.translation.y = centre.y;
                o.scale = scale;
                ctrl.zoom = scale; // keep resource in sync
            }

            if matches!(ctrl.mode, Mode::Recenter) {
                ctrl.mode = Mode::Free; // one‑shot complete
            }
        }
    }

    /* ── floating‑origin compatibility: translation already includes snap ─ */
    // The actual shift to keep numeric stability is applied later by
    // `apply_floating_origin`, which mutates *both* camera and world entities
    // and writes the cumulative displacement into `WorldOffset`.
    // Nothing to do here – we just ensure the transform is ready.

    // Z‑layer stays constant so we never cross UI‑layer depth.
    tf.translation.z = 1_000.0;
}
