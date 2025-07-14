use bevy::prelude::*;
use crate::ca_engine::{core::*, grid::GridBackend};

#[derive(Component)]
pub(crate) struct CellSprite;

pub fn paint_dense_board(
    mut commands:  Commands,
    world:         Res<World2D>,
    mut q_old:     Query<Entity, With<CellSprite>>,
) {
    for e in &mut q_old { commands.entity(e).despawn(); }

    let GridBackend::Dense(g) = &world.backend else { return };

    let half_w = g.size.x as f32 * world.cell_size * 0.5;
    let half_h = g.size.y as f32 * world.cell_size * 0.5;

    for (p, cell) in g.iter() {
        let colour = match cell.state {
            CellState::Dead     => Color::BLACK,
            CellState::Alive(n) => { let t = (n as f32) / 3.0; Color::srgb(t, 1.0 - t, 0.1) },
        };

        commands.spawn((
            Sprite {
                color: colour,
                custom_size: Some(Vec2::splat(world.cell_size - 1.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                p.x as f32 * world.cell_size - half_w,
                p.y as f32 * world.cell_size - half_h,
                0.0,
            )),
            GlobalTransform::default(),
            CellSprite,
        ));
    }
}