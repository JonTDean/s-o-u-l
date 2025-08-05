use bevy::{
    prelude::*,
    input::{
        ButtonInput,
        mouse::{MouseMotion, MouseButton},
        keyboard::KeyCode,
    },
    render::camera::Projection,
};

use super::systems::WorldCamera;

/* ========================================================================== */
/* Configuration                                                              */
/* ========================================================================== */

/// Default linear movement speed (**world-units · s⁻¹**).
pub const DEFAULT_MOVE_SPEED: f32 = 10.0;
/// Default mouse-look sensitivity (**radians · pixel⁻¹**).
pub const DEFAULT_LOOK_SENS:  f32 = 0.0025;

/// Run-time adjustable settings (eg. via Egui).
#[derive(Resource, Debug, Clone, Copy)]
pub struct FreeCamSettings {
    pub move_speed: f32,
    pub look_sens:  f32,
    pub require_mouse_button: Option<MouseButton>,
}
impl Default for FreeCamSettings {
    fn default() -> Self {
        Self {
            move_speed: DEFAULT_MOVE_SPEED,
            look_sens:  DEFAULT_LOOK_SENS,
            require_mouse_button: Some(MouseButton::Right),
        }
    }
}

/* ========================================================================== */
/* Components                                                                 */
/* ========================================================================== */

/// Marker for the single free-cam entity.
#[derive(Component, Debug)]
pub struct FreeCam;

/// Linear velocity accumulated by the input systems.
#[derive(Component, Debug, Default)]
struct Velocity(Vec3);

/// Yaw/pitch deltas accumulated by mouse-look.
#[derive(Component, Debug, Default)]
struct RotationDelta { yaw: f32, pitch: f32 }

/* ========================================================================== */
/* Spawner                                                                    */
/* ========================================================================== */

/// Spawns a perspective camera at (0, 0, 10) looking at the origin.
pub fn spawn_freecam(cmd: &mut Commands) {
    cmd.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4, // 45 °
            near: 0.1,
            far: 20_000.0,
            aspect_ratio: 1.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCam,
        WorldCamera,
        Velocity::default(),
        RotationDelta::default(),
    ));
}

/* ========================================================================== */
/* Stateless input systems                                                    */
/* ========================================================================== */

/// Collects WASD / Space / Q input and writes a world-space velocity.
fn gather_keyboard_input(
    keys:      Res<'_, ButtonInput<KeyCode>>,
    settings:  Res<'_, FreeCamSettings>,
    time:      Res<'_, Time>,
    mut q_cam: Query<'_, '_, (&mut Velocity, &Transform), With<FreeCam>>,
) {
    let Ok((mut vel, tf)) = q_cam.single_mut() else { return };

    let forward = tf.forward();
    let right   = tf.right();
    let up      = Vec3::Y;

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW)  { dir += *forward; }
    if keys.pressed(KeyCode::KeyS)  { dir -= *forward; }
    if keys.pressed(KeyCode::KeyD)  { dir += *right;   }
    if keys.pressed(KeyCode::KeyA)  { dir -= *right;   }
    if keys.pressed(KeyCode::Space) { dir += up;      }
    if keys.pressed(KeyCode::KeyQ)  { dir -= up;      }

    vel.0 = if dir != Vec3::ZERO {
        dir.normalize() * settings.move_speed * time.delta_secs()
    } else {
        Vec3::ZERO
    };
}

/// Translates mouse movement into yaw / pitch deltas.
fn gather_mouse_input(
    buttons:   Res<'_, ButtonInput<MouseButton>>,
    mut motion: EventReader<'_, '_, MouseMotion>,
    settings:  Res<'_, FreeCamSettings>,
    mut q_cam: Query<'_, '_, &mut RotationDelta, With<FreeCam>>,
) {
    if let Some(btn) = settings.require_mouse_button {
        if !buttons.pressed(btn) { return; }
    }

    let delta: Vec2 = motion.read().map(|m| m.delta).sum();
    if delta == Vec2::ZERO { return; }

    let Ok(mut rot) = q_cam.single_mut() else { return };

    rot.yaw   += -delta.x * settings.look_sens;
    rot.pitch += -delta.y * settings.look_sens;

    const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.017_453;
    rot.pitch = rot.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
}

/* ========================================================================== */
/* Heavy integrator – applies queued deltas                                   */
/* ========================================================================== */

fn apply_freecam_motion(
    mut q_cam: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut RotationDelta
        ),
        With<FreeCam>,
    >,
) {
    let Ok((mut tf, mut vel, mut rot)) = q_cam.single_mut() else { return };

    /* ── rotation ─────────────────────────────────────────────────── */
    if rot.yaw != 0.0 || rot.pitch != 0.0 {
        let (mut yaw, mut pitch, _) = tf.rotation.to_euler(EulerRot::YXZ);
        yaw   += rot.yaw;
        pitch += rot.pitch;
        tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        *rot = RotationDelta::default();
    }

    /* ── translation ──────────────────────────────────────────────── */
    if vel.0 != Vec3::ZERO {
        tf.translation += vel.0;
        *vel = Velocity::default();
    }
}

/* ========================================================================== */
/* Plugin                                                                     */
/* ========================================================================== */

/// Drop-in free-cam plugin (disabled until spawned).
pub struct FreeCamPlugin;
impl Plugin for FreeCamPlugin {
    fn build(&self, app: &mut App) {
        use super::CameraSet;
        use engine_core::systems::state::AppState;

        app.init_resource::<FreeCamSettings>()
            .add_systems(
                Update,
                (
                    gather_keyboard_input.in_set(CameraSet::Input),
                    gather_mouse_input.in_set(CameraSet::Input),
                )
                .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                apply_freecam_motion
                    .in_set(CameraSet::Heavy)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
