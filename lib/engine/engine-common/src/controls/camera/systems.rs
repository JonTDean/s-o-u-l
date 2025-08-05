//! World- & UI-camera manager (orthographic by default but 3-D-ready).
//!
//! *No panning / zooming / follow logic lives here* – that is provided by
//! [`super::controller`] so you can replace it at run-time if needed
//! (editor, cut-scenes, network sync…).

use bevy::{
    prelude::*,
    render::{
        camera::{OrthographicProjection, Projection, ScalingMode},
        view::RenderLayers,
    },
};
use engine_core::prelude::AppState;

use super::{
    input::{
        apply_orbit, begin_drag, drag_pan, end_drag, gather_orbit_input, key_pan, zoom_keyboard,
        zoom_scroll,
    },
    freecam::FreeCamPlugin,
};

/* ───────────────────────────── diagnostics ────────────────────────── */

/// World-space rectangle covered by the active viewport (debug overlay).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportRect {
    pub min: Vec2,
    pub max: Vec2,
}

/// Global camera diagnostics – updated every frame by `update_camera_metrics`.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CameraMetrics {
    pub centre:    Vec2,
    pub zoom:      f32,
    pub half_view: Vec2,
}

/* ───────────────────────── constants / helpers ────────────────────── */

pub const UI_LAYER:    u8 = 0;
pub const WORLD_LAYER: u8 = 1;

#[inline] pub fn layers_ui()    -> RenderLayers { RenderLayers::layer(UI_LAYER.into()) }
#[inline] pub fn layers_world() -> RenderLayers { RenderLayers::layer(WORLD_LAYER.into()) }

pub const KEY_PAN_SPEED:   f32 = 400.0;
pub const MIN_SCALE_CONST: f32 = 0.05;
pub const MAX_SCALE:       f32 = 128.0;
pub const ZOOM_FACTOR:     f32 = 1.1;

/* ───────────────────────── shared ECS types ───────────────────────── */

#[derive(Component)]           pub struct WorldCamera;
#[derive(Resource, Default)]   pub struct DragState(pub Option<Vec2>);
#[derive(Resource, Clone, Copy)]
pub struct ZoomInfo { pub base: f32, pub current: f32 }
impl Default for ZoomInfo { fn default() -> Self { Self { base: 1.0, current: 1.0 } } }

/* ───────────────────────── internal system sets ───────────────────── */

#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
pub enum CameraSet { Input, Heavy }

/* ───────────────────────── top-level plugin ───────────────────────── */

/// Manages the low-level orthographic camera(s) plus metrics refresh.
pub struct CameraManagerPlugin;
impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        /* register resources read / written by our systems */
        app.init_resource::<ZoomInfo>()
           .init_resource::<DragState>()
           .init_resource::<CameraMetrics>()

           /* create deterministic stages */
           .configure_sets(Update, (CameraSet::Input, CameraSet::Heavy.after(CameraSet::Input)))

           /* spawners */
           .add_systems(Startup, spawn_cameras)

           /* lightweight input (orthographic editor cam) */
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

           /* heavy work – metrics + orbit application */
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

           .add_plugins(FreeCamPlugin);
    }
}

/* ───────────────────────── helper systems ─────────────────────────── */

/// Sync `ZoomInfo.current` with the active orthographic projection.
fn refresh_zoom_info(cam_q: Query<&Projection, With<WorldCamera>>, mut z: ResMut<ZoomInfo>) {
    if let Ok(Projection::Orthographic(o)) = cam_q.single() {
        z.current = o.scale;
    }
}

/// Cache camera metrics once per frame; used by gizmos, minimap, etc.
fn update_camera_metrics(
    windows:  Query<&Window>,
    mut cam_q: Query<(&Transform, &Projection, &mut ViewportRect), With<WorldCamera>>,
    mut metrics: ResMut<CameraMetrics>,
) {
    let (Ok(win), Ok((tf, proj, mut rect))) = (windows.single(), cam_q.single_mut()) else { return };

    let ortho_scale = match proj {
        Projection::Orthographic(o) => { metrics.zoom = o.scale; o.scale },
        _                           => 1.0,
    };

    let half_view = Vec2::new(win.physical_width() as f32, win.physical_height() as f32)
        * 0.5 * ortho_scale;

    *metrics = CameraMetrics {
        centre: tf.translation.truncate(),
        zoom:   ortho_scale,
        half_view,
    };

    rect.min = metrics.centre - half_view;
    rect.max = metrics.centre + half_view;
}

/* ───────────────────────── camera spawner ─────────────────────────── */

/// Creates the UI + World camera pair (hidden until activated).
pub(crate) fn spawn_cameras(mut cmd: Commands, mut z: ResMut<ZoomInfo>) {
    z.base = 1.0; z.current = 1.0;

    /* UI camera – 2-D overlay (Egui, sprite HUD, etc.) */
    cmd.spawn((
        Camera2d::default(),
        Camera { order: 100, clear_color: ClearColorConfig::None, ..default() },
        layers_ui(),
    ));

    /* World camera – 3-D orthographic, inactive by default */
    let mut ortho = OrthographicProjection::default_3d();
    ortho.scaling_mode = ScalingMode::WindowSize;
    cmd.spawn((
        Camera3d::default(),
        Projection::Orthographic(ortho),
        Transform::from_xyz(0.0, 0.0, 1_000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera { order: 2, is_active: false, clear_color: ClearColorConfig::None, ..default() },
        WorldCamera,
        Visibility::Hidden,
        ViewportRect::default(),
        layers_world(),
    ));
}

/* ───────────────────────── AABB fit / clamp helper ────────────────── */
/// Compute `(centre, scale)` so the rectangle `world_min..=world_max`
/// is fully visible *and* clamped if smaller than the viewport.
#[allow(clippy::too_many_arguments)]
pub fn fit_or_clamp_camera(
    world_min: Vec3,
    world_max: Vec3,
    win:       &Window,
    _centre_in: Vec3,
    mut scale: f32,
) -> (Vec3, f32) {
    let mut centre = (world_min + world_max) * 0.5;

    /* 1 ░ pick a scale that fits once */
    let needed = ((world_max - world_min)
        / Vec3::new(win.width(), win.height(), 0.0))
        .max_element();
    scale = scale.max(needed).clamp(MIN_SCALE_CONST, MAX_SCALE);

    /* 2 ░ viewport extents in world units */
    let half_view = Vec3::new(win.width(), win.height(), 0.0) * 0.5 * scale;

    /* 3 ░ clamp **only if** the world is wider/higher than the viewport */
    let world_size = world_max - world_min;
    if half_view.x * 2.0 <= world_size.x {
        centre.x = centre.x.clamp(world_min.x + half_view.x,
                                   world_max.x - half_view.x);
    }
    if half_view.y * 2.0 <= world_size.y {
        centre.y = centre.y.clamp(world_min.y + half_view.y,
                                   world_max.y - half_view.y);
    }
    (centre, scale)
}
