use bevy::prelude::*;

use super::super::systems::{WorldCamera, KEY_PAN_SPEED};

/// WASD / arrow-key panning.
pub fn key_pan(
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

    let Ok((mut tf, proj)) = q.single_mut() else { return };
    let scale = match proj { Projection::Orthographic(o) => o.scale, _ => 1.0 };
    tf.translation += dir.normalize().extend(0.0) * KEY_PAN_SPEED * time.delta_secs() * scale;
}
