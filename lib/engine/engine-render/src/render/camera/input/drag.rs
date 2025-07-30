use bevy::prelude::*;

use super::super::systems::{DragState, WorldCamera};

/// Start a right / middle mouse drag.
pub fn begin_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
) {
    if buttons.just_pressed(MouseButton::Right) || buttons.just_pressed(MouseButton::Middle) {
        drag.0 = windows.single().ok().and_then(|w| w.cursor_position());
    }
}

/// Continue the drag → pan the camera.
pub fn drag_pan(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    if !(buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle)) { return; }
    let (Some(prev), Ok(win)) = (drag.0, windows.single()) else { return };
    let Some(cur) = win.cursor_position() else { return };
    let Ok((mut tf, proj)) = cam_q.single_mut() else { return };

    let scale = match proj { Projection::Orthographic(o) => o.scale, _ => 1.0 };
    let delta = cur - prev;
    tf.translation.x -= delta.x * scale;
    tf.translation.y -= delta.y * scale;
    drag.0 = Some(cur);
}

/// End the drag.
pub fn end_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
) {
    if buttons.just_released(MouseButton::Right) || buttons.just_released(MouseButton::Middle) {
        drag.0 = None;
    }
}
