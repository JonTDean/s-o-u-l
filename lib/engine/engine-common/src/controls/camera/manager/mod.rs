//! World- & UI-camera **manager** (orthographic by default, 3-D ready).
//!
//! *All* low-level glue lives here: camera spawning, diagnostics
//! refresh, built-in orbit/zoom/drag helpers, etc.  
//! Higher-level behaviour such as panning / follow lives in
//! [`super::controller`].

use bevy::{
    prelude::*,
    render::{
        camera::{OrthographicProjection, Projection, ScalingMode},
        view::RenderLayers,
    },
};


use super::input::{
        apply_orbit, begin_drag, drag_pan, end_drag, gather_orbit_input, key_pan,
        zoom_keyboard, zoom_scroll,
    };

/* ───────────────────────────── diagnostics ────────────────────────── */

/// World-space rectangle covered by the active viewport (debug overlay).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportRect {
    /// Lower-left world-space corner.
    pub min: Vec2,
    /// Upper-right world-space corner.
    pub max: Vec2,
}

/// Metrics refreshed once per frame by [`update_camera_metrics`].
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CameraMetrics {
    /// World-space centre of the view.
    pub centre:    Vec2,
    /// Current orthographic scale (**world-units / pixel**).
    pub zoom:      f32,
    /// Half-size of the viewport in world units.
    pub half_view: Vec2,
}
/* ───────────────────────── constants / helpers ────────────────────── */

/// Render layer used exclusively for 2-D UI/HUD elements.
pub const UI_LAYER:    u8 = 0;
/// Render layer reserved for world geometry & game entities.
pub const WORLD_LAYER: u8 = 1;

/// Mask containing only [`UI_LAYER`].
#[inline] pub fn layers_ui()    -> RenderLayers { RenderLayers::layer(UI_LAYER.into()) }
/// Mask containing only [`WORLD_LAYER`].
#[inline] pub fn layers_world() -> RenderLayers { RenderLayers::layer(WORLD_LAYER.into()) }

/// Linear keyboard-pan speed (**world-units · s⁻¹**).
pub const KEY_PAN_SPEED:   f32 = 400.0;
/// Minimum orthographic scale (world-units per screen-pixel).
pub const MIN_SCALE_CONST: f32 = 0.05;
/// Maximum orthographic scale (world-units per screen-pixel).
pub const MAX_SCALE:       f32 = 128.0;
/// Exponential zoom factor (≈ 10 % per wheel-tick).
pub const ZOOM_FACTOR:     f32 = 1.1;

/* ───────────────────── shared ECS types ──────────────────────────── */


/// Struct for world camera component
#[derive(Component)]           

pub struct WorldCamera;
#[derive(Resource, Default)]
/// Mouse-drag state shared between input systems.
pub struct DragState(pub Option<Vec2>);

/// Stores the original “boot” zoom and the current live zoom.
#[derive(Resource, Clone, Copy)]
pub struct ZoomInfo {
    /// Zoom level when the camera was spawned.
    pub base:     f32,
    /// Current zoom level.
    pub current:  f32,
}

impl Default for ZoomInfo { fn default() -> Self { Self { base: 1.0, current: 1.0 } } }


/* ───────────────────────── internal system sets ───────────────────── */

/// Sub-stages used by the camera stack.
#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
pub enum CameraSet {
    /// Runs *before* heavy maths – gathers input only.
    Input,
    /// Expensive computations (orbit integrator, metrics refresh…).
    Heavy,
}


/* ───────────────────────── helper systems ─────────────────────────── */

/// Sync [`ZoomInfo::current`] with the active orthographic projection.
pub fn refresh_zoom_info(cam_q: Query<&Projection, With<WorldCamera>>, mut z: ResMut<ZoomInfo>) {
    if let Ok(Projection::Orthographic(o)) = cam_q.single() {
        z.current = o.scale;
    }
}

/// Cache camera metrics once per frame; used by gizmos, minimap, etc.
pub fn update_camera_metrics(
    windows:     Query<&Window>,
    mut cam_q:   Query<(&Transform, &Projection, &mut ViewportRect), With<WorldCamera>>,
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

/// Creates the **single** UI camera (if missing) and the inactive world-camera.
///
/// * Called once during `Startup`.
/// * Safe even if another plugin (egui, a debug overlay, etc.) already added
///   its own 2-D camera – we simply re-use that one instead of spawning ours.\
/// 
/// args:
///     existing_ui: Query<Entity, With<Camera2d>>,  /// All cameras that *already* exist when we run.
pub fn spawn_cameras(
    mut cmd: Commands,
    mut z:   ResMut<ZoomInfo>,
    existing_ui: Query<Entity, With<Camera2d>>,
) {
    z.base = 1.0;
    z.current = 1.0;

    // ── UI camera ─────────────────────────────────────────────────────
    if existing_ui.iter().next().is_none() {
        cmd.spawn((
            Camera2d::default(),
            // Use a *unique* order so we never clash with other UI cameras.
            Camera { order: 50, clear_color: ClearColorConfig::None, ..default() },
            layers_ui(),
        ));
    }

    // ── World camera (inactive until InGame) ──────────────────────────

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

/* ─────────────────────── AABB fit / clamp helper ──────────────────── */

/// Compute *(centre, scale)* so that `world_min..=world_max` is fully
/// visible *and* clamped if smaller than the viewport.
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

/* ───────────────────────── sub-module: plugin ─────────────────────── */

pub mod plugin;
