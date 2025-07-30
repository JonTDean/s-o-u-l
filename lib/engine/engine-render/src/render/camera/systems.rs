//! -------------------------------------------------------------------------------------------------
//! World- & UI-camera manager (2025-07-30 refactor, **compile-fixed**).
//!
//! *   Inline closure in `.add_systems(OnEnter(..))` replaced by `send_recenter_on_enter()`.
//! *   All `send_default()` calls renamed to `write_default()` per
//!     <https://bevy.org/learn/migration-guides/0-15-to-0-16/>.
//!
//! Controls (unchanged)
//! --------------------
//! • Mouse wheel / drag / WASD – free-roam.  
//! • **F** – toggle *Follow* ↔ *Free*.  
//! • **C** – instant recenter without changing mode.
//! -------------------------------------------------------------------------------------------------

use bevy::{
    prelude::*,
    render::{camera::Projection, view::RenderLayers},
};
use engine_core::prelude::{AppState, AutomataRegistry};
use simulation_kernel::grid::GridBackend;
use tooling::debugging::camera::CameraDebug;

use crate::render::camera::{floating_origin::WorldOffset, gizmos::draw_camera_gizmos};

use super::{
    floating_origin::apply_floating_origin,
    input::{
        begin_drag, drag_pan, end_drag, key_pan, zoom_keyboard, zoom_scroll,
    },
};

/* ───────── Public constants, markers & resources ───────── */

pub const UI_LAYER:   u8 = 0;
pub const WORLD_LAYER: u8 = 1;

#[inline] pub fn layers_ui()    -> RenderLayers { RenderLayers::layer(UI_LAYER.into()) }
#[inline] pub fn layers_world() -> RenderLayers { RenderLayers::layer(WORLD_LAYER.into()) }

pub const ZOOM_FACTOR:     f32 = 1.1;
pub const MIN_SCALE_CONST: f32 = 0.05;
pub const MAX_SCALE:       f32 = 128.0;
pub const KEY_PAN_SPEED:   f32 = 400.0;

#[derive(Component)]              pub struct WorldCamera;
#[derive(Resource, Default)]      pub struct DragState(pub Option<Vec2>);
#[derive(Resource, Clone, Copy)]  pub struct ZoomInfo { pub base: f32, pub current: f32 }
impl Default for ZoomInfo { fn default() -> Self { Self { base: 1.0, current: 1.0 } } }

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum CameraTrackMode { Free, Follow }
impl Default for CameraTrackMode { fn default() -> Self { Self::Free } }

#[derive(Event, Default)] pub struct RecenterCamera;

/* ───────── Internal helpers ───────── */

#[derive(Component)] enum CameraKind { Ui, World }

#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
enum CameraSet { Input, Heavy }

/* ───────── Plugin ───────── */

pub struct CameraManagerPlugin;
impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            /* resources & events */
            .init_resource::<CameraDebug>()
            .init_resource::<ZoomInfo>()
            .init_resource::<DragState>()
            .init_resource::<WorldOffset>()
            .init_resource::<CameraTrackMode>()
            .add_event::<RecenterCamera>()

            /* system sets */
            .configure_sets(Update, (CameraSet::Input, CameraSet::Heavy.after(CameraSet::Input)))

            /* spawn cameras */
            .add_systems(Startup, spawn_cameras)

            /* state transitions */
            .add_systems(OnEnter(AppState::InGame),   activate_world_camera)
            .add_systems(OnEnter(AppState::MainMenu), ui_camera_enable_clear)
            .add_systems(OnExit (AppState::MainMenu), ui_camera_disable_clear)
            .add_systems(OnEnter(AppState::InGame),   send_recenter_on_enter)

            /* input & light work */
            .add_systems(
                Update,
                (
                    zoom_scroll,
                    zoom_keyboard,
                    begin_drag,
                    drag_pan,
                    end_drag,
                    key_pan,
                    toggle_track_mode,
                    request_recenter_key,
                )
                    .in_set(CameraSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )

            /* heavy work */
            .add_systems(
                Update,
                (
                    recenter_on_event,
                    follow_automata,
                    apply_floating_origin,
                    refresh_zoom_info,
                    draw_camera_gizmos,
                )
                    .in_set(CameraSet::Heavy)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/* ───────── 1 – spawn cameras ───────── */

pub(crate) fn spawn_cameras(mut cmd: Commands) {
    cmd.spawn((
        Camera2d,
        Camera { order: 100, clear_color: ClearColorConfig::None, ..default() },
        CameraKind::Ui,
        layers_ui(),
    ));
    cmd.spawn((
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 1_000.0)),
        Camera { order: 2, is_active: false, ..default() },
        CameraKind::World,
        WorldCamera,
        Visibility::Hidden,
        layers_world(),
    ));
}

/* ───────── 2 – menu ⇆ game toggles (unchanged) ───────── */

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
            CameraKind::Ui => { cam.clear_color = ClearColorConfig::Default; *vis = Visibility::Inherited; }
            CameraKind::World => { cam.is_active = false; *vis = Visibility::Hidden; }
        }
    }
}
fn ui_camera_disable_clear(mut q: Query<(&CameraKind, &mut Camera)>) {
    for (kind, mut cam) in &mut q {
        if matches!(kind, CameraKind::Ui) { cam.clear_color = ClearColorConfig::None; }
    }
}

/* ───────── 3-a – input helpers ───────── */

fn toggle_track_mode(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<CameraTrackMode>) {
    if keys.just_pressed(KeyCode::KeyF) {
        *mode = match *mode { CameraTrackMode::Free => CameraTrackMode::Follow, CameraTrackMode::Follow => CameraTrackMode::Free };
    }
}
fn request_recenter_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: EventWriter<RecenterCamera>,
) {
    if keys.just_pressed(KeyCode::KeyC) { writer.write_default(); }
}

/* ───────── 3-b – one-shot recenter helpers ───────── */

/// Fires once when entering *InGame*.
fn send_recenter_on_enter(mut ev: EventWriter<RecenterCamera>) {
    ev.write_default();
}
/* ───────────────────── 3-c. Event-driven recentre ───────────────────────── */

/// Immediately frames *all* automata when a [`RecenterCamera`] event is received.
///
/// Runs in `CameraSet::Heavy` because it touches the whole `AutomataRegistry`.
fn recenter_on_event(
    mut ev: EventReader<RecenterCamera>,
    windows: Query<&Window>,
    registry: Res<AutomataRegistry>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    if ev.is_empty() { return; }
    ev.clear(); // we only need to handle it once per frame

    let (Ok(win), Ok((mut tf, mut proj))) = (windows.single(), cam_q.single_mut()) else { return };
    if registry.list().is_empty() { return; }

    /* world-space AABB of all slices */
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);

    for info in registry.list() {
        let off  = info.world_offset.as_vec2() * info.cell_size;
        let size = match &info.grid {
            GridBackend::Dense(g)  => Vec2::new(g.size.x as f32, g.size.y as f32) * info.cell_size,
            GridBackend::Sparse(_) => Vec2::splat(512.0) * info.cell_size,
        };
        min = min.min(off);
        max = max.max(off + size);
    }

    if let Projection::Orthographic(ref mut ortho) = *proj {
        /* choose scale so the whole AABB fits, plus small padding */
        let pad = 2.0;
        let needed = (((max - min) + Vec2::splat(pad)) / Vec2::new(win.width(), win.height())).max_element();
        ortho.scale = needed.clamp(MIN_SCALE_CONST, MAX_SCALE);

        /* centre translation */
        let height   = max.y - min.y;
        let width = max.x - min.x;
        let view_h   = win.height() * ortho.scale;
        let view_w = win.width() * ortho.scale;
        let center_y = (min.y + max.y) * 0.5;
        let center_x = (min.x + max.x) * 0.5;
        
        // For Height
        if height <= view_h {
            tf.translation.y = center_y;          // free-roam inside the box on y
        } else {
            let half_h = view_h * 0.5;
            tf.translation.y = center_y.clamp(min.y + half_h, max.y - half_h);
        }
        
        // For width
        if width <= view_w {
            tf.translation.x = center_x         // free-roam inside the box on x
        } else {
            let half_w = view_w * 0.5;
            tf.translation.x = center_x.clamp(min.x + half_w, max.x - half_w);
        }
    }
}

/* ───────────────────── 3-d. Follow mode (optional) ─────────────────────── */

/// Re-applies the centring each frame *only* when `CameraTrackMode::Follow`.
fn follow_automata(
    mode: Res<CameraTrackMode>,
    windows: Query<&Window>,
    registry: Res<AutomataRegistry>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    if *mode != CameraTrackMode::Follow { return; }

    let (Ok(win), Ok((mut tf, mut proj))) = (windows.single(), cam_q.single_mut()) else { return };
    if registry.list().is_empty() { return; }

    /* (same bounding-box maths as recenter_on_event) */
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);

    for info in registry.list() {
        let off  = info.world_offset.as_vec2() * info.cell_size;
        let size = match &info.grid {
            GridBackend::Dense(g)  => Vec2::new(g.size.x as f32, g.size.y as f32) * info.cell_size,
            GridBackend::Sparse(_) => Vec2::splat(512.0) * info.cell_size,
        };
        min = min.min(off);
        max = max.max(off + size);
    }

    if let Projection::Orthographic(ref mut ortho) = *proj {
        let needed = ((max - min) / Vec2::new(win.width(), win.height())).max_element();
        ortho.scale = ortho.scale.max(needed).clamp(MIN_SCALE_CONST, MAX_SCALE);

        let half_w = win.width()  * 0.5 * ortho.scale;
        let half_h = win.height() * 0.5 * ortho.scale;

        tf.translation.x = ((min.x + max.x) * 0.5).clamp(min.x + half_w, max.x - half_w);
        tf.translation.y = ((min.y + max.y) * 0.5).clamp(min.y + half_h, max.y - half_h);
    }
}

/* ───────────────────── 3-e. Zoom resource sync ─────────────────────────── */

fn refresh_zoom_info(
    cam_q: Query<&Projection, With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    if let Ok(Projection::Orthographic(o)) = cam_q.single() {
        zoom.current = o.scale;
    }
}
