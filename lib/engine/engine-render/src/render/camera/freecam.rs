//! Perspective *free‑camera* for S.O.U.L. render‑world (Bevy ≥0.13).
//!
//! This **refactored** version fixes a bug where mouse‑look never triggered
//! because the `MouseMotion` events were ignored unless the right button
//! was pressed.  You can now choose between
//!
//! * **Hold‑to‑look** (default, RMB) – same behaviour as before.
//! * **Free‑look** (always active) – set `require_mouse_button` to `None` at
//!   run‑time and the camera will rotate continuously with mouse movement.
//!
//! # Usage
//!
//! ```rust
//! use engine_render::camera::freecam::{FreeCamPlugin, FreeCamSettings};
//!
//! App::new()
//!     .add_plugins((FreeCamPlugin, /* … */))
//!     .insert_resource(
//!         FreeCamSettings {
//!             move_speed: 12.0,
//!             look_sens:  0.0025,
//!             require_mouse_button: None, // ← free‑look all the time
//!         }
//!     )
//!     .run();
//! ```
//!
//! © 2025 Obaven Inc. • MIT License.

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
/* Run‑time configuration                                                     */
/* ========================================================================== */

/// Default linear movement speed (**world‑units · s⁻¹**).
pub const DEFAULT_MOVE_SPEED: f32 = 10.0;
/// Default mouse‑look sensitivity (**radians · pixel⁻¹**).
pub const DEFAULT_LOOK_SENS:  f32 = 0.0025;

/// Change these fields **at run‑time** (eg. from an egui panel) without
/// touching any systems.  The struct is a Bevy `Resource`, so reads are
/// inexpensive and fully thread‑safe.
///
/// * `move_speed` – metres‑per‑second when input is saturated (WASD).
/// * `look_sens` – yaw/pitch radians added **per screen pixel**.
/// * `require_mouse_button` –
///     * `Some(MouseButton::Right)` → **hold‑to‑look** (default),
///     * `None`                    → **free‑look** (mouse moves always rotate).
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

/// Marker for the single free‑cam entity produced by [`spawn_freecam`].
#[derive(Component, Debug)]
pub struct FreeCam;

/// Linear velocity accumulated by the *input* systems (reset every frame).
#[derive(Component, Debug, Default)]
struct Velocity(Vec3);

/// Yaw/pitch deltas accumulated by mouse‑look (reset every frame).
#[derive(Component, Debug, Default)]
struct RotationDelta { yaw: f32, pitch: f32 }

/* ========================================================================== */
/* Spawner                                                                    */
/* ========================================================================== */

/// Spawns a perspective camera at *(0, 0, 10)* looking toward the origin.
///
/// The entity is tagged with:
/// * [`FreeCam`]      – unique identifier for the query filters,
/// * [`WorldCamera`]  – lets the rest of the engine (HUD, gizmos, etc.)
///                      treat it as the *active* world camera.
pub fn spawn_freecam(cmd: &mut Commands) {
    cmd.spawn((
        // Camera core ------------------------------------------------------------------
        Camera3d::default(),
        // Perspective projection --------------------------------------------------------
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4, // 45 °
            near: 0.1,
            far: 20_000.0,
            aspect_ratio: 1.0,                // auto‑updated by Bevy
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
/* Stateless *input* systems (may run in parallel)                            */
/* ========================================================================== */

/// Collects WASD/Space/Q input and writes a *world‑space* velocity.
///
/// *The system is totally stateless.*  It performs no work when there is
/// no input and exhibits no side‑effects other than updating the `Velocity`
/// component, so Bevy is free to schedule it on any thread.
fn gather_keyboard_input(
    keys:      Res<'_, ButtonInput<KeyCode>>,
    settings:  Res<'_, FreeCamSettings>,
    time:      Res<'_, Time>,
    mut q_cam: Query<'_, '_, (&mut Velocity, &Transform), With<FreeCam>>,
) {
    let Ok((mut vel, tf)) = q_cam.single_mut() else { return };

    // Local axes -----------------------------------------------------------------------
    let forward: Vec3 = tf.forward().into(); // +Z *‑1 by convention
    let right:   Vec3 = tf.right().into();
    let up              = Vec3::Y;

    // Direction from keyboard state ----------------------------------------------------
    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW)     { dir += forward; }
    if keys.pressed(KeyCode::KeyS)     { dir -= forward; }
    if keys.pressed(KeyCode::KeyD)     { dir += right;   }
    if keys.pressed(KeyCode::KeyA)     { dir -= right;   }
    if keys.pressed(KeyCode::Space)    { dir += up;      }
    if keys.pressed(KeyCode::KeyQ)     { dir -= up;      }

    // Convert to *per‑frame* displacement (Δ = v · dt) --------------------------------
    vel.0 = if dir != Vec3::ZERO {
        dir.normalize() * settings.move_speed * time.delta_secs()
    } else {
        Vec3::ZERO
    };
}

/// Translates raw mouse movement (Δ‑pixels) into yaw/pitch deltas.
///
/// The function is entirely self‑contained and only mutates the *local*
/// `RotationDelta` component – no global state, perfect work for
/// parallel‑execution.
fn gather_mouse_input(
    buttons:   Res<'_, ButtonInput<MouseButton>>,
    mut motion: EventReader<'_, '_, MouseMotion>,
    settings:  Res<'_, FreeCamSettings>,
    mut q_cam: Query<'_, '_, &mut RotationDelta, With<FreeCam>>,
) {
    // Early‑out when “hold‑to‑look” is enabled but the button isn’t held.
    if let Some(btn) = settings.require_mouse_button {
        if !buttons.pressed(btn) { return; }
    }

    // Accumulate all motion events from this frame.
    let delta: Vec2 = motion.read().map(|m| m.delta).sum();
    if delta == Vec2::ZERO { return; }

    let Ok(mut rot) = q_cam.single_mut() else { return };

    rot.yaw   += -delta.x * settings.look_sens;
    rot.pitch += -delta.y * settings.look_sens;
    // Clamp pitch to avoid gimbal lock / upside‑down camera.
    const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.017_453; // 89 °
    rot.pitch = rot.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
}

/* ========================================================================== */
/* Heavy system – applies the queued deltas                                   */
/* ========================================================================== */

/// Applies the accumulated `Velocity` and `RotationDelta` to the `Transform`.
///
/// The system mutates *only one entity*, so Bevy will normally schedule it
/// on the main thread to avoid overhead, but nothing here requires that.
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

    /* ───── rotation ─────────────────────────────────────────────────────────── */
    if rot.yaw != 0.0 || rot.pitch != 0.0 {
        // Extract current Euler angles in the same YXZ order we will write back.
        let (mut yaw, mut pitch, _) = tf.rotation.to_euler(EulerRot::YXZ);
        yaw   += rot.yaw;
        pitch += rot.pitch;

        // Pitch is already clamped in `gather_mouse_input`, keep yaw unbounded.
        tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);

        // Reset the accumulator so we don’t apply it twice.
        *rot = RotationDelta::default();
    }

    /* ───── translation ─────────────────────────────────────────────────────── */
    if vel.0 != Vec3::ZERO {
        tf.translation += vel.0;
        *vel = Velocity::default();
    }
}

/* ========================================================================== */
/* Plugin                                                                     */
/* ========================================================================== */

/// Drop‑in plugin – **just add it to your `App`**.
///
/// All systems are slotted into the same internal `CameraSet` stages used
/// by the orthographic camera manager so the execution order is
/// deterministic and free of double‑borrow hazards.
pub struct FreeCamPlugin;
impl Plugin for FreeCamPlugin {
    fn build(&self, app: &mut App) {
        use super::systems::CameraSet; // keeps the ordering consistent
        use engine_core::systems::state::AppState;

        app.init_resource::<FreeCamSettings>()
            // Spawn the camera as soon as we enter the game scene.
            .add_systems(OnEnter(AppState::InGame), |mut cmd: Commands| {
                spawn_freecam(&mut cmd);
            })
            // Lightweight input collectors ------------------------------------------------
            .add_systems(
                Update,
                (
                    gather_keyboard_input.in_set(CameraSet::Input),
                    gather_mouse_input.in_set(CameraSet::Input),
                )
                .run_if(in_state(AppState::InGame)),
            )
            // Heavy integrator ------------------------------------------------------------
            .add_systems(
                Update,
                apply_freecam_motion
                    .in_set(CameraSet::Heavy)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
