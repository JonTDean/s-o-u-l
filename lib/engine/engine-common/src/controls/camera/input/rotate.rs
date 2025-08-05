use bevy::{input::mouse::MouseMotion, prelude::*};
use super::super::systems::WorldCamera;

/* ─────────────────────────── resource ───────────────────────────── */
#[derive(Resource, Default)]
pub struct OrbitAngles {
    pub yaw:   f32,   // rotation around +Z (world “up”)
    pub pitch: f32,   // rotation around +X (local)
}

/* ─────────────────────── input collection ───────────────────────── */
pub fn gather_orbit_input(
    mut angles: ResMut<OrbitAngles>,
    buttons:    Res<ButtonInput<MouseButton>>,
    keys:       Res<ButtonInput<KeyCode>>,
    mut motion: EventReader<MouseMotion>,
) {
    if !(buttons.pressed(MouseButton::Left) && keys.pressed(KeyCode::AltLeft)) {
        return;
    }
    let delta: Vec2 = motion.read().map(|m| m.delta).sum();
    if delta == Vec2::ZERO { return; }

    angles.yaw   -= delta.x * 0.005;
    angles.pitch = (angles.pitch - delta.y * 0.005).clamp(-1.55, 1.55); // ±89 °
}

/* ─────────────────────── application ────────────────────────────── */
pub fn apply_orbit(
    angles: Res<OrbitAngles>,
    mut cam_q: Query<&mut Transform, With<WorldCamera>>,
) {
    let Ok(mut tf) = cam_q.single_mut() else { return };

    let radius = tf.translation.length();
    tf.translation = Quat::from_euler(EulerRot::YXZ, angles.yaw, angles.pitch, 0.0)
        * Vec3::new(0.0, 0.0, radius);

    tf.look_at(Vec3::ZERO, Vec3::Y);
}
