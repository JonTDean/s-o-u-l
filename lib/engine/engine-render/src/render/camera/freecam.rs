//! Perspective *free‑camera* for S.O.U.L. render‑world (Bevy ≥0.13).
//!
//! A **self‑contained** first‑person camera that lets you *fly* through the 3‑D
//! scene with **W / A / S / D** plus *Space/Shift* for vertical motion and
//! mouse‑look while the **right mouse button** is held.
//!
//! * Independent of the orthographic stack – disable it without side‑effects.
//! * Tagged with [`WorldCamera`] so HUD, floating‑origin & gizmos keep working.
//! * Spawns automatically on `AppState::InGame`, or manually via
//!   [`spawn_freecam`].
//!
//! ```no_run
//! use engine_render::camera::freecam::{FreeCamPlugin, spawn_freecam};
//! # use bevy::prelude::*;
//! App::new()
//!     .add_plugins((/* other plugins */, FreeCamPlugin))
//!     .add_systems(Startup, |mut cmd: Commands| {
//!         // Manual spawn (optional)
//!         spawn_freecam(&mut cmd);
//!     })
//!     .run();
//! ```
//!
//! © 2025 Obaven Inc. • MIT License.

use bevy::{
    prelude::*,
    input::{ButtonInput, mouse::{MouseMotion, MouseButton}, keyboard::KeyCode},
    render::camera::Projection,
};

use super::systems::WorldCamera; // keep overlays happy

/* ========================================================================== */
/* Config                                                                     */
/* ========================================================================== */

/// Default movement speed (**world‑units · s⁻¹**).
pub const DEFAULT_MOVE_SPEED: f32 = 10.0;
/// Default mouse‑look sensitivity (**radians · pixel⁻¹**).
pub const DEFAULT_LOOK_SENS:  f32 = 0.0025;

/// Run‑time tweakable settings.
#[derive(Resource, Debug, Clone, Copy)]
pub struct FreeCamSettings {
    pub move_speed: f32,
    pub look_sens:  f32,
}
impl Default for FreeCamSettings {
    fn default() -> Self {
        Self { move_speed: DEFAULT_MOVE_SPEED, look_sens: DEFAULT_LOOK_SENS }
    }
}

/* ========================================================================== */
/* Components                                                                 */
/* ========================================================================== */

/// Marker for the single free‑cam entity.
#[derive(Component, Debug)]
pub struct FreeCam;

/// Per‑frame translation Δ.
#[derive(Component, Debug, Default)]
struct Velocity(Vec3);
/// Per‑frame yaw/pitch Δ (radians).
#[derive(Component, Debug, Default)]
struct RotationDelta { yaw_pitch: Vec2 }

/* ========================================================================== */
/* Spawner                                                                    */
/* ========================================================================== */

/// Creates a perspective camera at *(0,0,10)* looking towards the origin.
pub fn spawn_freecam(cmd: &mut Commands) {
    cmd.spawn((
        // Camera core ------------------------------------------------------------------
        Camera3d::default(),
        // Perspective projection --------------------------------------------------------
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4, // 45°
            near: 0.1,
            far: 20_000.0,
            aspect_ratio: 1.0,                // updated by Bevy
            ..default()
        }),
        // Transform ---------------------------------------------------------------------
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Tags & state ------------------------------------------------------------------
        FreeCam,
        WorldCamera,
        Velocity::default(),
        RotationDelta::default(),
    ));
}

/* ========================================================================== */
/* Input collection (stateless, light‑weight)                                 */
/* ========================================================================== */

fn gather_keyboard_input(
    keys:      Res<ButtonInput<KeyCode>>,
    settings:  Res<FreeCamSettings>,
    time:      Res<Time>,
    mut q_cam: Query<(&mut Velocity, &Transform), With<FreeCam>>,
) {
    let Ok((mut vel, tf)) = q_cam.single_mut() else { return };

    // Convert Dir3 → Vec3 because +/‑ with Vec3 isn’t defined.
    let fwd:   Vec3 = tf.forward().into(); // –Z axis
    let right: Vec3 = tf.right().into();
    let up           = Vec3::Y;

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { dir += fwd;   }
    if keys.pressed(KeyCode::KeyS) { dir -= fwd;   }
    if keys.pressed(KeyCode::KeyD) { dir += right; }
    if keys.pressed(KeyCode::KeyA) { dir -= right; }
    if keys.pressed(KeyCode::Space) { dir += up;   }
    if keys.pressed(KeyCode::KeyQ)  { dir -= up;   }

    vel.0 = if dir != Vec3::ZERO {
        dir.normalize() * settings.move_speed * time.delta_secs()
    } else {
        Vec3::ZERO
    };
}

fn gather_mouse_input(
    buttons:   Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    settings:  Res<FreeCamSettings>,
    mut q_cam: Query<&mut RotationDelta, With<FreeCam>>,
) {
    if !buttons.pressed(MouseButton::Right) { return; }
    let delta: Vec2 = motion.read().map(|m| m.delta).sum();
    if delta == Vec2::ZERO { return; }
    let Ok(mut rot) = q_cam.single_mut() else { return };

    rot.yaw_pitch += Vec2::new(-delta.x, -delta.y) * settings.look_sens;
}

/* ========================================================================== */
/* Motion application (heavy)                                                 */
/* ========================================================================== */

fn apply_freecam_motion(
    mut q_cam: Query<(&mut Transform, &mut Velocity, &mut RotationDelta), With<FreeCam>>,
) {
    let Ok((mut tf, mut vel, mut rot)) = q_cam.single_mut() else { return };

    // Rotation -------------------------------------------------------------------------
    if rot.yaw_pitch != Vec2::ZERO {
        const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.017_453; // 89°
        let (_, mut yaw, mut pitch) = tf.rotation.to_euler(EulerRot::YXZ);
        yaw   += rot.yaw_pitch.x;
        pitch += rot.yaw_pitch.y;
        pitch  = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        rot.yaw_pitch = Vec2::ZERO;
    }

    // Translation ----------------------------------------------------------------------
    if vel.0 != Vec3::ZERO {
        tf.translation += vel.0;
        vel.0 = Vec3::ZERO;
    }
}

/* ========================================================================== */
/* Plugin                                                                     */
/* ========================================================================== */

/// Drop‑in Bevy plugin.
pub struct FreeCamPlugin;
impl Plugin for FreeCamPlugin {
    fn build(&self, app: &mut App) {
        use super::systems::CameraSet; // keep Input / Heavy ordering consistent
        use engine_core::systems::state::AppState;

        app.init_resource::<FreeCamSettings>()
            .add_systems(OnEnter(AppState::InGame), |mut cmd: Commands| {
                spawn_freecam(&mut cmd);
            })
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
