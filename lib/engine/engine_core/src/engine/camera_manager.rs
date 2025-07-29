//! Two‑camera (UI + world) stack with proper clear‑colour switching and
//! RTS‑style zoom / pan controls.  Compatible with Bevy ≥ 0.16.

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::{camera::Projection, view::RenderLayers},
};

use crate::{
    engine::{
        components::{WorldCamera, ZoomInfo},
        worldgrid::WorldGrid,
    },
    state::AppState,
};

/* ── Layers ────────────────────────────────────────────────────────── */

pub const UI_LAYER:    u8 = 0;
pub const WORLD_LAYER: u8 = 1;

#[inline]
fn layers_ui()    -> RenderLayers { RenderLayers::layer(UI_LAYER.into()) }
#[inline]
fn layers_world() -> RenderLayers { RenderLayers::layer(WORLD_LAYER.into()) }

/* ── Tuning constants ──────────────────────────────────────────────── */

const ZOOM_FACTOR:   f32 = 1.1;
const MIN_SCALE:     f32 = 0.05;
const MAX_SCALE:     f32 = 128.0;
const KEY_PAN_SPEED: f32 = 400.0;

/* ── Internals ─────────────────────────────────────────────────────── */

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum CameraKind { Ui, World }

#[derive(Resource, Default)]
struct DragState(Option<Vec2>);

/* ── Plugin ────────────────────────────────────────────────────────── */

pub struct CameraManagerPlugin;

impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomInfo>()
            .init_resource::<DragState>()
            /* 1 ─ spawn the two cameras once */
            .add_systems(Startup, spawn_cameras)
            /* 2 ─ camera (de)activation & clear‑colour management */
            .add_systems(OnEnter(AppState::InGame),   activate_world_camera)
            .add_systems(OnEnter(AppState::MainMenu), ui_camera_enable_clear)
            .add_systems(OnExit (AppState::MainMenu), ui_camera_disable_clear)
            /* 3 ─ recentre once the WorldGrid exists */
            .add_systems(OnEnter(AppState::InGame), centre_on_world)
            /* 4 ─ in‑game controls */
            .add_systems(
                Update,
                (
                    zoom_scroll,
                    zoom_keyboard,
                    begin_drag,
                    drag_pan,
                    end_drag,
                    key_pan,
                    refresh_zoom_info,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/* ── 1. Spawn cameras ──────────────────────────────────────────────── */

fn spawn_cameras(mut cmd: Commands) {
    /* UI camera – draws last, normally *does not* clear */
    cmd.spawn((
        Camera2d,
        Camera {
            order:       100,
            clear_color: ClearColorConfig::None,
            is_active:   true,
            ..default()
        },
        CameraKind::Ui,
        layers_ui(),
    ));

    /* World camera – enabled during gameplay */
    cmd.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 1_000.0)),
        Camera {
            order:     2,
            is_active: false,
            ..default()
        },
        CameraKind::World,
        WorldCamera,
        Visibility::Hidden,
        layers_world(),
    ));
}

/* ── 2‑a. Activate world camera on game entry (keep UI camera on) ──── */

fn activate_world_camera(mut q: Query<(&CameraKind, &mut Visibility, &mut Camera)>) {
    for (kind, mut vis, mut cam) in &mut q {
        if matches!(kind, CameraKind::World) {
            cam.is_active = true;
            *vis = Visibility::Inherited;
        }
        // UI camera stays untouched (remains active)
    }
}

/* ── 2‑b. Menu background handling – toggle clear‑colour  ──────────── */

/// Entering the main menu: make UI cam clear to the default clear colour
/// and shut the world camera off to avoid wasted draws.
fn ui_camera_enable_clear(
    mut q: Query<(&CameraKind, &mut Camera, &mut Visibility)>,
) {
    for (kind, mut cam, mut vis) in &mut q {
        match kind {
            CameraKind::Ui => {
                cam.is_active  = true;
                cam.clear_color = ClearColorConfig::Default; // solid background
                *vis = Visibility::Inherited;
            }
            CameraKind::World => {
                cam.is_active = false;
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Leaving the main menu: restore “transparent overlay” mode.
fn ui_camera_disable_clear(
    mut q: Query<(&CameraKind, &mut Camera)>,
) {
    for (kind, mut cam) in &mut q {
        if matches!(kind, CameraKind::Ui) {
            cam.clear_color = ClearColorConfig::None;
        }
    }
}

/* ── 3. Centre world camera on the active grid ─────────────────────── */

fn centre_on_world(
    grid: Option<Res<WorldGrid>>,
    mut tf: Query<&mut Transform, With<WorldCamera>>,
) {
    let Ok(mut tf) = tf.single_mut() else { return };
    let Some(grid) = grid else { return };

    let (w, h) = match &grid.backend {
        crate::engine::grid::GridBackend::Dense(g)  => (g.size.x, g.size.y),
        crate::engine::grid::GridBackend::Sparse(_) => (1_024, 1_024),
    };
    tf.translation.x = w as f32 * 0.5;
    tf.translation.y = h as f32 * 0.5;
}

/* ── 4‑a. Zoom & pan controls ─────────────────────────────────────── */

fn zoom_scroll(
    mut wheel: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    let (mut tf, mut proj) = if let Ok(v) = cam_q.single_mut() { v } else { return };
    let scroll: f32 = wheel.read().map(|e| e.y).sum();
    if scroll == 0.0 { return; }

    let Some(win) = windows.iter().next() else { return };
    let Some(cursor) = win.cursor_position() else { return };

    let win_size   = Vec2::new(win.width(), win.height());
    let cursor_ndc = (cursor / win_size) * 2.0 - Vec2::ONE;

    if let Projection::Orthographic(ref mut o) = *proj {
        let world_before = tf.translation.truncate()
            + cursor_ndc * (win_size * 0.5) * o.scale;

        o.scale = (o.scale * if scroll > 0.0 { 1.0 / ZOOM_FACTOR } else { ZOOM_FACTOR })
            .clamp(MIN_SCALE, MAX_SCALE);
        zoom.current = o.scale;

        let world_after = tf.translation.truncate()
            + cursor_ndc * (win_size * 0.5) * o.scale;

        tf.translation += (world_before - world_after).extend(0.0);
    }
}

fn zoom_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam_q: Query<&mut Projection, With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    let mut proj = if let Ok(p) = cam_q.single_mut() { p } else { return };
    if !(keys.just_pressed(KeyCode::KeyZ) ^ keys.just_pressed(KeyCode::KeyX)) { return; }
    if let Projection::Orthographic(ref mut o) = *proj {
        if keys.just_pressed(KeyCode::KeyZ) { o.scale /= ZOOM_FACTOR; }
        if keys.just_pressed(KeyCode::KeyX) { o.scale *= ZOOM_FACTOR; }
        o.scale = o.scale.clamp(MIN_SCALE, MAX_SCALE);
        zoom.current = o.scale;
    }
}

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
    if !(buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle)) { return; }
    let (Some(prev), Ok(win)) = (drag.0, windows.single()) else { return };
    let Some(cur) = win.cursor_position() else { return };

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

fn key_pan(
    keys:  Res<ButtonInput<KeyCode>>,
    time:  Res<Time>,
    mut q:  Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }

    if dir == Vec2::ZERO { return; }
    let (mut tf, proj) = if let Ok(v) = q.single_mut() { v } else { return };
    let scale = match proj { Projection::Orthographic(o) => o.scale, _ => 1.0 };

    tf.translation += dir.normalize().extend(0.0)
        * KEY_PAN_SPEED * time.delta_secs() * scale;
}

/* ── 4‑b. Sync zoom overlay ────────────────────────────────────────── */

fn refresh_zoom_info(
    cam_q: Query<&Projection, With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    if let Ok(Projection::Orthographic(o)) = cam_q.single() {
        zoom.current = o.scale;
    }
}

