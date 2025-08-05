//! Zoom helpers – scroll-wheel & keyboard shortcuts.
//!
//! Keeps mouse-centric zoom behaviour in sync with [`ZoomInfo`] so other
//! systems (minimap, gizmos…) can read the live orthographic scale.

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::Projection,
};

use crate::controls::camera::manager::{
    WorldCamera, ZoomInfo, ZOOM_FACTOR, MIN_SCALE_CONST, MAX_SCALE,
};

/// Mouse-wheel zoom (centred under the cursor).
pub fn zoom_scroll(
    mut wheel: EventReader<MouseWheel>,
    windows:  Query<&Window>,
    mut cam_q: Query<(&mut Transform, &mut Projection), With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    let scroll: f32 = wheel.read().map(|e| e.y).sum();
    if scroll == 0.0 { return; }

    let (Ok((mut tf, mut proj)), Some(win)) = (cam_q.single_mut(), windows.iter().next()) else { return };
    let Some(cursor) = win.cursor_position() else { return };

    let win_size   = Vec2::new(win.width(), win.height());
    let cursor_ndc = (cursor / win_size) * 2.0 - Vec2::ONE;

    if let Projection::Orthographic(ref mut o) = *proj {
        let world_before = tf.translation.truncate()
            + cursor_ndc * (win_size * 0.5) * o.scale;

        o.scale = (o.scale * if scroll > 0.0 { 1.0 / ZOOM_FACTOR } else { ZOOM_FACTOR })
            .clamp(MIN_SCALE_CONST, MAX_SCALE);
        zoom.current = o.scale;

        let world_after = tf.translation.truncate()
            + cursor_ndc * (win_size * 0.5) * o.scale;

        tf.translation += (world_before - world_after).extend(0.0);
    }
}

/// Keyboard zoom (`Z` / `X`).
pub fn zoom_keyboard(
    keys:  Res<ButtonInput<KeyCode>>,
    mut cam:  Query<&mut Projection, With<WorldCamera>>,
    mut zoom: ResMut<ZoomInfo>,
) {
    if !(keys.just_pressed(KeyCode::KeyZ) ^ keys.just_pressed(KeyCode::KeyX)) { return; }
    let Ok(mut proj) = cam.single_mut() else { return };
    if let Projection::Orthographic(ref mut o) = *proj {
        if keys.just_pressed(KeyCode::KeyZ) { o.scale /= ZOOM_FACTOR; }
        if keys.just_pressed(KeyCode::KeyX) { o.scale *= ZOOM_FACTOR; }
        o.scale = o.scale.clamp(MIN_SCALE_CONST, MAX_SCALE);
        zoom.current = o.scale;
    }
}
