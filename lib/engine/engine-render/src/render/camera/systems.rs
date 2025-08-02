//! World- & UI-camera manager – **stripped of all 2-D clamps / follow code**.
use bevy::{
    prelude::*,
    render::{
        camera::{Projection, OrthographicProjection, ScalingMode},
        view::RenderLayers,
    },
};
use engine_core::prelude::AppState;
use tooling::debugging::camera::CameraDebug;

use crate::render::camera::{freecam::FreeCamPlugin, input::{apply_orbit, gather_orbit_input, OrbitAngles}};

use super::{
    floating_origin::{apply_floating_origin, WorldOffset},
    input::{begin_drag, drag_pan, end_drag, key_pan, zoom_keyboard, zoom_scroll},
};

/* ───────────────────────────── public diagnostics ───────────────────────── */

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ViewportRect { pub min: Vec2, pub max: Vec2 }

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CameraMetrics {
    pub centre:    Vec2,
    pub zoom:      f32,
    pub half_view: Vec2,
}

/* ───────────────────────────── constants / helpers ───────────────────────── */

pub const UI_LAYER:    u8 = 0;
pub const WORLD_LAYER: u8 = 1;

#[inline] pub fn layers_ui()    -> RenderLayers { RenderLayers::layer(UI_LAYER.into()) }
#[inline] pub fn layers_world() -> RenderLayers { RenderLayers::layer(WORLD_LAYER.into()) }

pub const KEY_PAN_SPEED:    f32 = 400.0;
pub const MIN_SCALE_CONST:  f32 = 0.05;
pub const MAX_SCALE:        f32 = 128.0;
pub const ZOOM_FACTOR:      f32 = 1.1;

#[derive(Component)]           pub struct WorldCamera;
#[derive(Resource, Default)]   pub struct DragState(pub Option<Vec2>);
#[derive(Resource, Clone, Copy)]
pub struct ZoomInfo { pub base: f32, pub current: f32 }
impl Default for ZoomInfo { fn default() -> Self { Self { base: 1.0, current: 1.0 } } }

/* ───────────────────────────── internal sets ────────────────────────────── */

#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
pub(crate) enum CameraSet { Input, Heavy }

/* ───────────────────────────── top-level plugin ─────────────────────────── */

pub struct CameraManagerPlugin;
impl Plugin for CameraManagerPlugin {
    fn build(&self, app: &mut App) {
        /* register *all* resources that systems read or write */
        app.init_resource::<CameraDebug>()
            .init_resource::<WorldOffset>()
            .init_resource::<ZoomInfo>()
            .init_resource::<DragState>()
            .init_resource::<CameraMetrics>()
            .init_resource::<OrbitAngles>()

            /* sets & spawners */
            .configure_sets(Update, (CameraSet::Input, CameraSet::Heavy.after(CameraSet::Input)))
            .add_systems(Startup, spawn_cameras)

            /* lightweight input (still useful for the orthographic editor cam) */
            .add_systems(
                Update,
                (
                    // Zoom configs
                    zoom_scroll, zoom_keyboard,
                    // Drag configs
                    begin_drag, drag_pan, end_drag,
                    // Pan configs
                    key_pan,
                    // Orbital configs
                    gather_orbit_input,  
                )
                    .in_set(CameraSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )

            /* heavy – floating origin & metrics only (no more clamps) */
            .add_systems(
                Update,
                (
                    apply_floating_origin,
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

/* ───────────────────────────── helper systems ───────────────────────────── */

fn refresh_zoom_info(cam_q: Query<&Projection, With<WorldCamera>>, mut z: ResMut<ZoomInfo>) {
    if let Ok(Projection::Orthographic(o)) = cam_q.single() {
        z.current = o.scale;
    }
}

fn update_camera_metrics(
    windows: Query<&Window>,
    mut cam_q: Query<(&Transform, &Projection, &mut ViewportRect), With<WorldCamera>>, // <-- &mut
    mut metrics: ResMut<CameraMetrics>,
) {
    let (Ok(win), Ok((tf, proj, mut rect))) = (windows.single(), cam_q.single_mut()) else { return };
    let ortho_scale = matches!(proj, Projection::Orthographic(o) if { metrics.zoom = o.scale; true })
        .then_some(())
        .map(|_| if let Projection::Orthographic(o) = proj { o.scale } else { 1.0 })
        .unwrap_or(1.0);

    let half_view = Vec2::new(win.physical_width() as f32, win.physical_height() as f32)
        * 0.5 * ortho_scale;

    *metrics = CameraMetrics { centre: tf.translation.truncate(), zoom: ortho_scale, half_view };
    rect.min = metrics.centre - half_view;
    rect.max = metrics.centre + half_view;
}

/* ───────────────────────────── camera spawner ───────────────────────────── */

pub(crate) fn spawn_cameras(mut cmd: Commands, mut z: ResMut<ZoomInfo>) {
    z.base = 1.0; z.current = 1.0;

    /* UI camera (unchanged) */
    cmd.spawn((
        Camera2d::default(),
        Camera { order: 100, clear_color: ClearColorConfig::None, ..default() },
        layers_ui(),
    ));

    /* World camera – still provide a 3-D ortho for any editor tools */
    let mut ortho = OrthographicProjection::default_3d();
    ortho.scaling_mode = ScalingMode::WindowSize;
    cmd.spawn((
        Camera3d::default(),
        Projection::Orthographic(ortho),
        Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera { order: 2, is_active: false, clear_color: ClearColorConfig::None, ..default() },
        WorldCamera,
        Visibility::Hidden,
        ViewportRect::default(),
        layers_world(),
    ));
}

/* ───────────────────────────────────────────────────────────────────── */
/* CAMERA FIT / CLAMP HELPER                                            */
/* ───────────────────────────────────────────────────────────────────── */
/// Returns updated `(centre, scale)` that guarantees the AABB is visible.
///
///   We now seed the computation with the **exact centre of `world_min..max`**
///   instead of the incoming transform position.  This means a single
///   “recentre / C-key” will always land the camera perfectly in the middle
///   and the yellow bounds will frame the view on *all* four sides.
pub fn fit_or_clamp_camera(
    world_min: Vec3,
    world_max: Vec3,
    win:       &Window,
    _centre_in: Vec3,
    mut scale: f32,
) -> (Vec3, f32) {
    let mut centre = (world_min + world_max) * 0.5;

    // 1 ░ pick a scale that fits once
    let needed = ((world_max - world_min)
        / Vec3::new(win.width(), win.height(), 0.0))
        .max_element();
    scale = scale.max(needed).clamp(MIN_SCALE_CONST, MAX_SCALE);

    // 2 ░ viewport extents in world units
    let half_view = Vec3::new(win.width(), win.height(), 0.0) * 0.5 * scale;

    // 3 ░ clamp **only if** the world is wider/higher than the viewport
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
