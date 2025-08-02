use bevy::prelude::*;
use super::super::systems::{WorldCamera, DragState};


/// Alt + left-mouse → orbit around the scene-origin.
pub fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    keys:    Res<ButtonInput<KeyCode>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
    mut cam_q: Query<&mut Transform, With<WorldCamera>>,
) {
    // only when Alt is held and L-MB is pressed
    if !(buttons.pressed(MouseButton::Left) && keys.pressed(KeyCode::AltLeft)) {
        return;
    }

    let window = match windows.single() { Ok(w) => w, Err(_) => return };

    /* ── start drag ───────────────────────────────────────── */
    if drag.0.is_none() {
        drag.0 = window.cursor_position();
        return;
    }

    /* ── continue drag ────────────────────────────────────── */
    let prev = drag.0.unwrap();
    let Some(cur) = window.cursor_position() else { return };
    let delta = cur - prev;
    drag.0 = Some(cur);

    let Ok(mut tf) = cam_q.single_mut() else { return };

    // horizontal: yaw around global +Z   ––––––––––––––––––––––––
    tf.rotate_around(Vec3::ZERO, Quat::from_rotation_z(-delta.x * 0.005));
    // vertical:   pitch around local +X  ––––––––––––––––––––––––
    tf.rotate_local_x(-delta.y * 0.005);
}

/// Clear drag state on L-MB release.
pub fn end_orbit(buttons: Res<ButtonInput<MouseButton>>, mut drag: ResMut<DragState>) {
    if buttons.just_released(MouseButton::Left) {
        drag.0 = None;
    }
}