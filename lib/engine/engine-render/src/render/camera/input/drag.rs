use bevy::prelude::*;

use super::super::systems::{DragState, WorldCamera};

/// Start right / middle mouse drag.
pub fn begin_drag(
    buttons: Res<ButtonInput<MouseButton>>, mut drag: ResMut<DragState>, windows: Query<&Window>,
) {
    if buttons.just_pressed(MouseButton::Right) || buttons.just_pressed(MouseButton::Middle) {
        drag.0 = windows.single().ok().and_then(|w| w.cursor_position());
    }
}

/// Continue drag → pan camera in world units (zoom‑corrected).
pub fn drag_pan(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    if !(buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle)) { return; }
    let (Some(prev), Ok(win)) = (drag.0, windows.single()) else { return }; // previous cursor pos
    let Some(cur) = win.cursor_position() else { return };
    let Ok((mut tf, proj)) = cam_q.single_mut() else { return };

    if let Projection::Orthographic(o) = proj {
        let win_size = Vec2::new(win.width(), win.height());
        let ndc_prev = (prev / win_size) * 2.0 - Vec2::ONE;
        let ndc_cur  = (cur  / win_size) * 2.0 - Vec2::ONE;
        let delta_world = (ndc_cur - ndc_prev) * win_size * 0.5 * o.scale;
        tf.translation.x -= delta_world.x;
        tf.translation.y += delta_world.y;
    }
    drag.0 = Some(cur);
}

/// End drag.
pub fn end_drag(buttons: Res<ButtonInput<MouseButton>>, mut drag: ResMut<DragState>) {
    if buttons.just_released(MouseButton::Right) || buttons.just_released(MouseButton::Middle) {
        drag.0 = None;
    }
}