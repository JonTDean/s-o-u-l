//! RTS‑style zoom + pan for the world camera (Bevy 0.16.1).

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::Projection,          // brings the enum into scope
};

const ZOOM_FACTOR: f32 = 1.1;
const MIN_SCALE:  f32  = 0.1;
const MAX_SCALE:  f32  = 64.0;

/// Added to the world camera in `grid2d::setup`.
#[derive(Component)]
pub struct WorldCamera;

/* ─────────────── Zoom bookkeeping ─────────────── */

#[derive(Resource, Default)]             // <- `Default` needed by `init_resource`
pub struct ZoomInfo {
    pub base:    f32,    // start‑of‑scenario scale (set by `grid2d::setup`)
    pub current: f32,    // updated every time the user zooms
}

/* ─────────────── Internal helpers ─────────────── */

#[derive(Resource, Default)]
struct DragState(Option<Vec2>);

/* ─────────────── Plugin ─────────────── */

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
           .init_resource::<ZoomInfo>()
           .add_systems(
               Update,
               (
                   zoom_scroll,
                   zoom_keyboard,
                   begin_drag,
                   drag_pan,
                   end_drag,
               )
                   .run_if(in_state(engine_core::state::AppState::InGame)),
           );
    }
}

/* ─────────────────── Zoom – mouse wheel ─────────────────── */

fn zoom_scroll(
    mut wheel:  EventReader<MouseWheel>,
    mut cam_q:  Query<&mut Projection, With<WorldCamera>>,
    mut zoom:   ResMut<ZoomInfo>,
) {
    let Ok(mut proj) = cam_q.single_mut() else { return };

    let scroll: f32 = wheel.read().map(|e| e.y).sum();
    if scroll == 0.0 {
        return;
    }

    if let Projection::Orthographic(ref mut ortho) = *proj {
        let sign = scroll.signum();
        ortho.scale = (ortho.scale
            * if sign > 0.0 { 1.0 / ZOOM_FACTOR } else { ZOOM_FACTOR })
            .clamp(MIN_SCALE, MAX_SCALE);
        zoom.current = ortho.scale;
    }
}

/* ─────────────────── Zoom – keyboard (Z / X) ─────────────────── */

fn zoom_keyboard(
    keys:      Res<ButtonInput<KeyCode>>,
    mut cam_q: Query<&mut Projection, With<WorldCamera>>,
    mut zoom:  ResMut<ZoomInfo>,
) {
    let Ok(mut proj) = cam_q.single_mut() else { return };

    if let Projection::Orthographic(ref mut ortho) = *proj {
        if keys.just_pressed(KeyCode::KeyZ) {
            ortho.scale /= ZOOM_FACTOR;
        }
        if keys.just_pressed(KeyCode::KeyX) {
            ortho.scale *= ZOOM_FACTOR;
        }
        ortho.scale = ortho.scale.clamp(MIN_SCALE, MAX_SCALE);
        zoom.current = ortho.scale;
    }
}

/* ─────────────────── Pan (RMB drag) ─────────────────── */

fn begin_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows: Query<&Window>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        if let Ok(win) = windows.single() {
            drag.0 = win.cursor_position();
        }
    }
}

fn drag_pan(
    buttons:  Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    windows:  Query<&Window>,
    mut cam_q: Query<(&mut Transform, &Projection), With<WorldCamera>>,
) {
    if !buttons.pressed(MouseButton::Right) {
        return;
    }
    let Some(prev) = drag.0 else { return };
    let Ok(win)    = windows.single()      else { return };
    let Some(cur)  = win.cursor_position() else { return };

    drag.0 = Some(cur);

    let (mut tf, proj) = match cam_q.single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    let scale = match proj {
        Projection::Orthographic(o) => o.scale,
        _                           => 1.0,
    };

    let delta = cur - prev;
    tf.translation.x -= delta.x * scale;
    tf.translation.y += delta.y * scale;   // screen‑y up == world‑y+
}

fn end_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
) {
    if buttons.just_released(MouseButton::Right) {
        drag.0 = None;
    }
}
