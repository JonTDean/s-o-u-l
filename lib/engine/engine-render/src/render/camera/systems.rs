//! World- & UI-camera manager (2025-07-31 – compile-clean).

use bevy::{
    prelude::*,
    render::{camera::Projection, view::RenderLayers},
};
use engine_core::prelude::{AppState, AutomataRegistry};
use simulation_kernel::grid::{DenseGrid, GridBackend};
use tooling::debugging::camera::CameraDebug;

use crate::render::{
    camera::{floating_origin::WorldOffset, gizmos::draw_camera_gizmos},
    worldgrid::WorldGrid,
};

use super::{
    floating_origin::apply_floating_origin,
    input::{begin_drag, drag_pan, end_drag, key_pan, zoom_keyboard, zoom_scroll},
};

/* ───────────── Public constants & helpers ───────────── */

pub const UI_LAYER: u8 = 0;
pub const WORLD_LAYER: u8 = 1;

#[inline] pub fn layers_ui()    -> RenderLayers { RenderLayers::layer(UI_LAYER.into()) }
#[inline] pub fn layers_world() -> RenderLayers { RenderLayers::layer(WORLD_LAYER.into()) }

pub const ZOOM_FACTOR:     f32 = 1.1;
pub const MIN_SCALE_CONST: f32 = 0.05;
pub const MAX_SCALE:       f32 = 128.0;
pub const KEY_PAN_SPEED:   f32 = 400.0;

#[derive(Component)] pub struct WorldCamera;
#[derive(Resource, Default)] pub struct DragState(pub Option<Vec2>);
#[derive(Resource, Clone, Copy)]
pub struct ZoomInfo { pub base: f32, pub current: f32 }

impl Default for ZoomInfo {
    fn default() -> Self { Self { base: 1.0, current: 1.0 } }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum CameraTrackMode { Free, Follow }
impl Default for CameraTrackMode { fn default() -> Self { Self::Free } }

#[derive(Event, Default)] pub struct RecenterCamera;

/// Fit / clamp helper – returns updated `(centre, scale)`.
pub fn fit_or_clamp_camera(
    world_min: Vec2,
    world_max: Vec2,
    win: &Window,
    mut centre: Vec2,
    mut scale: f32,
) -> (Vec2, f32) {
    // 1 ░ ensure world fits at least once (C-key or initial recenter).
    let needed = ((world_max - world_min) / Vec2::new(win.width(), win.height())).max_element();
    scale = scale.max(needed).clamp(MIN_SCALE_CONST, MAX_SCALE);

    // 2 ░ slack when viewport bigger than world AABB.
    let half_view = Vec2::new(win.width(), win.height()) * 0.5 * scale;
    let slack     = (world_max - world_min) * 0.5 - half_view;

    // 3 ░ clamp each axis only if no slack left.
    if slack.x <= 0.0 {
        centre.x = centre.x.clamp(world_min.x + half_view.x, world_max.x - half_view.x);
    }
    if slack.y <= 0.0 {
        centre.y = centre.y.clamp(world_min.y + half_view.y, world_max.y - half_view.y);
    }

    (centre, scale)
}

/* ───────────── Internal sets & markers ───────────── */

#[derive(Component)] enum CameraKind { Ui, World }
#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)] enum CameraSet { Input, Heavy }

/* ───────────── Top-level plugin ───────────── */

pub struct CameraManagerPlugin;
impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDebug>()
           .init_resource::<ZoomInfo>()
           .init_resource::<DragState>()
           .init_resource::<WorldOffset>()
           .init_resource::<CameraTrackMode>()
           .add_event::<RecenterCamera>()
           .configure_sets(Update, (CameraSet::Input, CameraSet::Heavy.after(CameraSet::Input)))
           .add_systems(Startup, spawn_cameras) // now pub(crate)
           // state toggles
           .add_systems(OnEnter(AppState::InGame),   activate_world_camera)
           .add_systems(OnEnter(AppState::MainMenu), ui_camera_enable_clear)
           .add_systems(OnExit (AppState::MainMenu), ui_camera_disable_clear)
           .add_systems(OnEnter(AppState::InGame),   send_recenter_on_enter)
           // light-weight input set
           .add_systems(
               Update,
               (
                   zoom_scroll, zoom_keyboard,
                   begin_drag, drag_pan, end_drag,
                   key_pan,
                   toggle_track_mode, request_recenter_key,
               )
                   .in_set(CameraSet::Input)
                   .run_if(in_state(AppState::InGame)),
           )
           // heavy set (AABB queries, floating-origin, gizmos)
           .add_systems(
               Update,
               (
                   recenter_on_event,
                   follow_automata,
                   apply_floating_origin,
                   refresh_zoom_info,
                   draw_camera_gizmos,
                   clamp_camera_to_world,                  // ← adjusted
               )
                   .in_set(CameraSet::Heavy)
                   .run_if(in_state(AppState::InGame)),
           );
    }
}

/* ───────────── 1. Spawn cameras ───────────── */

pub(crate) fn spawn_cameras(mut cmd: Commands, mut zoom: ResMut<ZoomInfo>) {
    let ortho_start = 1.0;                 // start scale
    zoom.base = ortho_start;
    zoom.current = ortho_start;

    // UI camera (always active)
    cmd.spawn((Camera2d,
               Camera { order: 100, clear_color: ClearColorConfig::None, ..default() },
               CameraKind::Ui,
               layers_ui()));

    // World camera (activated only in-game)
    cmd.spawn((Camera2d,
               Transform::from_translation(Vec3::new(0.0, 0.0, 1_000.0)),
               Camera { order: 2, is_active: false, ..default() },
               CameraKind::World,
               WorldCamera,
               Visibility::Hidden,
               layers_world()));
}

/* ───────────── 2. Menu ↔ game toggles ───────────── */

fn activate_world_camera(mut q: Query<(&CameraKind, &mut Visibility, &mut Camera)>) {
    for (kind, mut vis, mut cam) in &mut q {
        if matches!(kind, CameraKind::World) {
            cam.is_active = true;
            *vis = Visibility::Inherited;
        }
    }
}
fn ui_camera_enable_clear(mut q: Query<(&CameraKind, &mut Camera, &mut Visibility)>) {
    for (kind, mut cam, mut vis) in &mut q {
        match kind {
            CameraKind::Ui    => { cam.clear_color = ClearColorConfig::Default; *vis = Visibility::Inherited; },
            CameraKind::World => { cam.is_active   = false;                    *vis = Visibility::Hidden; },
        }
    }
}
fn ui_camera_disable_clear(mut q: Query<(&CameraKind, &mut Camera)>) {
    for (kind, mut cam) in &mut q {
        if matches!(kind, CameraKind::Ui) { cam.clear_color = ClearColorConfig::None; }
    }
}

/* ───────────── 3-a. Input helpers ───────────── */

fn toggle_track_mode(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<CameraTrackMode>) {
    if keys.just_pressed(KeyCode::KeyF) {
        *mode = match *mode { CameraTrackMode::Free => CameraTrackMode::Follow, CameraTrackMode::Follow => CameraTrackMode::Free };
    }
}
fn request_recenter_key(keys: Res<ButtonInput<KeyCode>>, mut writer: EventWriter<RecenterCamera>) {
    if keys.just_pressed(KeyCode::KeyC) { writer.write_default(); }
}
fn send_recenter_on_enter(mut ev: EventWriter<RecenterCamera>) {
    ev.write_default();
}

/* ───────────── 3-b. One-shot recenter ───────────── */

fn recenter_on_event(
    mut ev: EventReader<RecenterCamera>,
    windows: Query<&Window>,
    registry: Res<AutomataRegistry>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    if ev.is_empty() {
        return;
    }
    ev.clear();

    let (Ok(win), Ok((mut tf, mut proj))) = (windows.single(), cam_q.single_mut()) else {
        return;
    };
    if registry.list().is_empty() {
        return;
    }

    /* compute world AABB */
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for info in registry.list() {
        let off = info.world_offset;
        let size = match &info.grid {
            GridBackend::Dense(g) => {
                Vec2::new(g.size.x as f32, g.size.y as f32) * info.cell_size
            }
            GridBackend::Sparse(_) => Vec2::splat(512.0) * info.cell_size,
        };
        min = min.min(off);
        max = max.max(off + size);
    }

    if let Projection::Orthographic(ref mut ortho) = *proj {
        let (centre, scale) =
            fit_or_clamp_camera(min, max, win, tf.translation.truncate(), ortho.scale);
        tf.translation.x = centre.x;
        tf.translation.y = centre.y;
        ortho.scale = scale;
    }
}

/* ───────────── 3-c. Follow mode ───────────── */

fn follow_automata(
    mode: Res<CameraTrackMode>,
    windows: Query<&Window>,
    registry: Res<AutomataRegistry>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    if *mode != CameraTrackMode::Follow {
        return;
    }

    let (Ok(win), Ok((mut tf, mut proj))) = (windows.single(), cam_q.single_mut()) else {
        return;
    };
    if registry.list().is_empty() {
        return;
    }

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for info in registry.list() {
        let off = info.world_offset;
        let size = match &info.grid {
            GridBackend::Dense(g) => {
                Vec2::new(g.size.x as f32, g.size.y as f32) * info.cell_size
            }
            GridBackend::Sparse(_) => Vec2::splat(512.0) * info.cell_size,
        };
        min = min.min(off);
        max = max.max(off + size);
    }

    if let Projection::Orthographic(ref mut ortho) = *proj {
        let (centre, scale) =
            fit_or_clamp_camera(min, max, win, tf.translation.truncate(), ortho.scale);
        tf.translation.x = centre.x;
        tf.translation.y = centre.y;
        ortho.scale = scale;
    }
}

/* ───────────── 3-d. Zoom resource sync ───────────── */

fn refresh_zoom_info(
    cam_q: Query<&Projection, With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    if let Ok(Projection::Orthographic(o)) = cam_q.single() {
        zoom.current = o.scale;
    }
}

/* ───────────────── 3-e. helper: keep camera inside the world ─────────────── */
fn clamp_camera_to_world(
    windows: Query<&Window>,
    world:   Option<Res<WorldGrid>>,
    mut cam_q: Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    let (Ok(win), Some(world)) = (windows.single(), world) else { return };
    let Ok((mut tf, proj)) = cam_q.single_mut() else { return };

    // orthographic scale (world-units per screen-pixel)
    let scale      = match proj { Projection::Orthographic(o) => o.scale, _ => 1.0 };
    let half_view  = Vec2::new(win.width(), win.height()) * 0.5 * scale;

    // 1 ░ original world size in *cell* units
    let (mut w_cells, mut h_cells) = match &world.backend {
        GridBackend::Dense(g)  => (g.size.x as f32, g.size.y as f32),
        GridBackend::Sparse(_) => (1024.0, 1024.0),
    };

    // 2 ░ ensure the logical bounds are always at least as large as the viewport
    //     → avoids min > max panics and lets us treat the current window as
    //       the definitive world when the grid is smaller.
    w_cells = w_cells.max(half_view.x * 2.0);
    h_cells = h_cells.max(half_view.y * 2.0);

    // 3 ░ clamp the camera centre so its viewport never leaves those bounds
    tf.translation.x = tf.translation.x.clamp(half_view.x, w_cells - half_view.x);
    tf.translation.y = tf.translation.y.clamp(half_view.y, h_cells - half_view.y);
}

/// Returns a rectangle that is always at least as big as the viewport.
pub(crate) fn dynamic_world_size(win: &Window, proj_scale: f32, grid: &DenseGrid) -> Vec2 {
    let half_view = Vec2::new(win.width(), win.height()) * 0.5 * proj_scale;
    Vec2::new(
        (grid.size.x as f32).max(half_view.x * 2.0),
        (grid.size.y as f32).max(half_view.y * 2.0),
    )
}
