use bevy::prelude::*;
use crate::ca_engine::{core::*, grid::GridBackend};

pub fn handle_mouse_clicks(
    buttons:  Res<ButtonInput<MouseButton>>,
    windows:  Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<World2D>,
) {
    if !buttons.just_pressed(MouseButton::Left) { return; }

    let Ok(window)                = windows.single() else { return };
    let Ok((cam, cam_xf))         = camera_q.single() else { return };
    let Some(cursor)              = window.cursor_position() else { return };
    let Ok(world_pos)             = cam.viewport_to_world_2d(cam_xf, cursor) else { return };

    let (size, cell_sz) = match &world.backend {
        GridBackend::Dense(g) => (g.size, world.cell_size),
        _ => return,
    };

    let half_w = size.x as f32 * cell_sz * 0.5;
    let half_h = size.y as f32 * cell_sz * 0.5;
    let gx = ((world_pos.x + half_w) / cell_sz).floor() as i32;
    let gy = ((world_pos.y + half_h) / cell_sz).floor() as i32;
    if gx < 0 || gy < 0 || gx >= size.x as i32 || gy >= size.y as i32 { return; }

    if let GridBackend::Dense(g) = &mut world.backend {
        let idx = (gy as u32 * size.x + gx as u32) as usize;
        let cell = &mut g.cells[idx];
        cell.state = match cell.state { CellState::Dead => CellState::Alive(1), _ => CellState::Dead };
    }
}