//! engine/components.rs
//! ====================
//! Runtime ECS components **and** low‑level helper resources that the
//! SOUL engine exposes to the rest of the application.  In particular
//! this file now hosts the entire *world‑camera* stack:
//
//!  * Camera spawning is driven by [`WorldGrid`] so the view is always
//!    centred on the active world‑atlas, regardless of which automata
//!    are alive.
//!  * RTS‑style zoom‑and‑pan input is handled locally; no dependency on
//!    `input::controls::camera_control` remains.
//!
//! ## Public surface
//! Add the plugin once during start‑up
//!
//! ```rust,no_run
//! use engine_core::engine::components::CameraPlugin;
//! app.add_plugins(CameraPlugin);
//! ```
//!
//! Everything – camera entity, zoom/pan resources – is created lazily
//! the first time a `WorldGrid` resource appears while **`AppState::InGame`**
//! is active.  This avoids the early‑initialisation panic that plagued
//! the old two‑plugin setup.

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::Projection,
};

use crate::{
    state::AppState,
    engine::worldgrid::WorldGrid,
};

/* ========================================================================= */
/*                               Configuration                               */
/* ========================================================================= */

/// Per‑“notch” zoom multiplier (and Z / X key‑step).
const ZOOM_FACTOR: f32 = 1.1;
/// Minimum and maximum orthographic scales.
const MIN_SCALE:  f32 = 0.05;
const MAX_SCALE:  f32 = 128.0;
/// Keyboard pan speed in **world** units s⁻¹ at scale == 1.0.
const KEY_PAN_SPEED: f32 = 400.0;

/* ========================================================================= */
/*                                 Components                                */
/* ========================================================================= */

/// Marker attached to the **single** world camera.
#[derive(Component)]
pub struct WorldCamera;

/* ========================================================================= */
/*                                  Resources                                */
/* ========================================================================= */

/// Current zoom factor (`ortho.scale`) – queried by render materials & UI.
#[derive(Resource, Clone, Copy)]
pub struct ZoomInfo {
    pub base:    f32,
    pub current: f32,
}

impl Default for ZoomInfo {
    fn default() -> Self {
        Self { base: 1.0, current: 1.0 }
    }
}

/// Mouse‑drag bookkeeping.
#[derive(Resource, Default)]
struct DragState(Option<Vec2>);

/* ========================================================================= */
/*                                    Plugin                                 */
/* ========================================================================= */

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomInfo>()
            .init_resource::<DragState>()
            .add_systems(
                Update,
                (
                    spawn_once_when_worldgrid_exists,
                    zoom_scroll,
                    zoom_keyboard,
                    begin_drag,
                    drag_pan,
                    end_drag,
                    key_pan,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/* ========================================================================= */
/*                       1 ── Lazy camera creation                           */
/* ========================================================================= */

fn spawn_once_when_worldgrid_exists(
    mut cmd:   Commands,
    mut done:  Local<bool>,
    grid_opt:  Option<Res<WorldGrid>>,
    mut zoom:  ResMut<ZoomInfo>,
) {
    if *done || grid_opt.is_none() { return; }
    let grid = grid_opt.unwrap();

    // Decide sensible initial focus ---------------------------------------
    let (w, h) = match &grid.backend {
        crate::engine::grid::GridBackend::Dense(g)  => (g.size.x, g.size.y),
        crate::engine::grid::GridBackend::Sparse(_) => (1024, 1024),
    };

    // NOTE:  we *could* carry a cell‑size inside WorldGrid; for now use 1.0
    //        because render quads apply their own scaling anyway.
    let half = Vec2::new(w as f32, h as f32) * 0.5;

    cmd.spawn((
        Camera2d,
        Camera { order: 2, ..default() },
        Transform::from_translation(half.extend(1000.0)),
        WorldCamera,
    ));
    zoom.base = 1.0;
    zoom.current = 1.0;
    *done = true;
}

/* ========================================================================= */
/*                       2 ── Zoom controls (mouse)                          */
/* ========================================================================= */

fn zoom_scroll(
    mut wheel: EventReader<MouseWheel>,
    windows:   Query<&Window>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
    mut zoom:  ResMut<ZoomInfo>,
) {
    let (mut tf, mut proj) = if let Ok(v) = cam_q.single_mut() { v } else { return };
    let scroll: f32 = wheel.read().map(|e| e.y).sum();
    if scroll == 0.0 { return; }

    let Some(win)           = windows.iter().next()         else { return };
    let Some(cursor_screen) = win.cursor_position()         else { return };

    // Screen‑space → NDC
    let win_size    = Vec2::new(win.width(), win.height());
    let cursor_ndc  = (cursor_screen / win_size) * 2.0 - Vec2::ONE;

    if let Projection::Orthographic(ref mut o) = *proj {
        // World pos under mouse BEFORE zoom
        let world_before = tf.translation.truncate()
            + cursor_ndc * (win_size * 0.5) * o.scale;

        // Modify scale
        o.scale = (o.scale
            * if scroll > 0.0 { 1.0 / ZOOM_FACTOR } else { ZOOM_FACTOR })
            .clamp(MIN_SCALE, MAX_SCALE);
        zoom.current = o.scale;

        // AFTER zoom
        let world_after = tf.translation.truncate()
            + cursor_ndc * (win_size * 0.5) * o.scale;

        tf.translation += (world_before - world_after).extend(0.0);
    }
}

/* ========================================================================= */
/*                       3 ── Zoom controls (keyboard)                       */
/* ========================================================================= */

fn zoom_keyboard(
    keys:     Res<ButtonInput<KeyCode>>,
    windows:  Query<&Window>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    if !(keys.just_pressed(KeyCode::KeyZ) ^ keys.just_pressed(KeyCode::KeyX)) {
        return;
    }
    let (mut tf, mut proj) = if let Ok(v) = cam_q.single_mut() { v } else { return };
    let win = if let Ok(w) = windows.single() { w } else { return };
    let win_size = Vec2::new(win.width(), win.height());

    if let Projection::Orthographic(ref mut o) = *proj {
        let world_before = tf.translation.truncate();

        if keys.just_pressed(KeyCode::KeyZ) { o.scale /= ZOOM_FACTOR; }
        if keys.just_pressed(KeyCode::KeyX) { o.scale *= ZOOM_FACTOR; }
        o.scale = o.scale.clamp(MIN_SCALE, MAX_SCALE);
        zoom.current = o.scale;

        let world_after = tf.translation.truncate();
        tf.translation += (world_before - world_after).extend(0.0);
    }
}

/* ========================================================================= */
/*                           4 ── Mouse‑drag pan                             */
/* ========================================================================= */

fn begin_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
) {
    if buttons.just_pressed(MouseButton::Right) || buttons.just_pressed(MouseButton::Middle) {
        if let Ok(win) = windows.single() {
            drag.0 = win.cursor_position();
        }
    }
}

fn drag_pan(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    if !(buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle)) {
        return;
    }
    let Some(prev) = drag.0 else { return };
    let Ok(win)    = windows.single() else { return };
    let Some(cur)  = win.cursor_position() else { return };

    let (mut tf, proj) = if let Ok(v) = cam_q.single_mut() { v } else { return };
    let scale = match proj { Projection::Orthographic(o) => o.scale, _ => 1.0 };

    let delta = cur - prev;
    tf.translation.x -= delta.x * scale;
    tf.translation.y += delta.y * scale;
    drag.0 = Some(cur);
}

fn end_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
) {
    if buttons.just_released(MouseButton::Right) || buttons.just_released(MouseButton::Middle) {
        drag.0 = None;
    }
}

/* ========================================================================= */
/*                        5 ── Keyboard‑pan (WASD)                           */
/* ========================================================================= */

fn key_pan(
    keys:    Res<ButtonInput<KeyCode>>,
    time:    Res<Time>,
    mut cam_q: Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }

    if dir == Vec2::ZERO { return; }
    let (mut tf, proj) = if let Ok(v) = cam_q.single_mut() { v } else { return };
    let scale = match proj { Projection::Orthographic(o) => o.scale, _ => 1.0 };

    tf.translation += dir.normalize().extend(0.0)
        * KEY_PAN_SPEED * time.delta_secs() * scale;
}
