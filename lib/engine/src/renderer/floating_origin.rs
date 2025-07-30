//! Floating‑origin maintenance.
//!
//! Keeps the [`WorldCamera`] near (0, 0) and shifts **everything else**
//! by whole‑multiples of `SNAP`.  Precision problems disappear while the
//! user perceives no discontinuity.

use bevy::prelude::*;

use super::components::WorldCamera;

/// Size of one “chunk” in world units.  
/// (= 256 cells if `cell_size == 1.0`)
const SNAP: i32 = 256;

/// Global world‑offset (in **world units**, already multiplied by
/// `cell_size`).  Used by render materials & GPU compute shaders.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct WorldOffset(pub IVec2);

/// System: run **after** any camera‑movement system.
pub fn apply_floating_origin(
    mut cam_q:  Query<&mut Transform, With<WorldCamera>>,
    mut world_q: Query<&mut Transform, Without<Camera>>,
    mut offset: ResMut<WorldOffset>,
) {
    let mut cam_tf = match cam_q.single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    // Has the camera strayed too far?
    if cam_tf.translation.x.abs() < SNAP as f32
        && cam_tf.translation.y.abs() < SNAP as f32
    {
        return;
    }

    // Whole‑chunk displacement (signed)
    let dx = (cam_tf.translation.x / SNAP as f32).round() as i32;
    let dy = (cam_tf.translation.y / SNAP as f32).round() as i32;
    let delta = Vec3::new(-(dx as f32) * SNAP as f32, -(dy as f32) * SNAP as f32, 0.0);

    // 1 ── recenter the camera
    cam_tf.translation += delta;

    // 2 ── shift *every* non‑camera transform the opposite way
    for mut tf in &mut world_q {
        tf.translation -= delta;
    }

    // 3 ── remember offset
    offset.0 += IVec2::new(dx * SNAP, dy * SNAP);
}
