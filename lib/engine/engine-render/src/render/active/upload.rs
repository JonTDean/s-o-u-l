//! Helper functions that copy the CPU grid into a GPU image each frame.
//! (These are currently unused by the new renderer but kept for optional
//! debug tooling / screenshots.)

use engine_core::world::World2D;
use glam::IVec2;
use bevy::prelude::*;
use simulation_kernel::{core::cell::CellState, grid::GridBackend};
use std::collections::HashSet;


use super::plugin::ActiveImageHandle;

#[derive(Resource, Default)]
pub struct PrevLive(pub HashSet<IVec2>);

/* --------------------------------------------------------------------- */

/// Dense‑grid upload (full rewrite every frame).
#[inline(always)]
pub fn upload_dense(
    world: Res<World2D>,
    mut images: ResMut<Assets<Image>>,
    image_handle: Res<ActiveImageHandle>,
) {
    if let GridBackend::Dense(grid) = &world.backend {
        if let Some(buf) = images
            .get_mut(&image_handle.0)
            .and_then(|img| img.data.as_mut())
            .filter(|b| b.len() == grid.cells.len())
        {
            for (idx, cell) in grid.cells.iter().enumerate() {
                buf[idx] = if matches!(cell.state, CellState::Dead) { 0 } else { 255 };
            }
        }
    }
}

/// Sparse‑grid upload (differential).
pub fn upload_sparse(
    world: Res<World2D>,
    mut images: ResMut<Assets<Image>>,
    image_handle: Res<ActiveImageHandle>,
    mut prev: ResMut<PrevLive>,
) {
    if let GridBackend::Sparse(sparse) = &world.backend {
        if let Some(image) = images.get_mut(&image_handle.0) {
            let tex_w = image.texture_descriptor.size.width as i32;
            let tex_h = image.texture_descriptor.size.height as i32;

            if let Some(buf) = image.data.as_mut() {
                let mut current_live = HashSet::new();

                /* gather live cells */
                for (&pos, cell) in sparse.map.iter() {
                    if !matches!(cell.state, CellState::Dead) {
                        current_live.insert(pos);
                    }
                }

                /* clear pixels that died */
                for &pos in &prev.0 {
                    if !current_live.contains(&pos)
                        && (0..tex_w).contains(&pos.x)
                        && (0..tex_h).contains(&pos.y)
                    {
                        let idx = (pos.y as u32 * tex_w as u32 + pos.x as u32) as usize;
                        buf[idx] = 0;
                    }
                }

                /* set pixels that were born */
                for &pos in &current_live {
                    if !prev.0.contains(&pos)
                        && (0..tex_w).contains(&pos.x)
                        && (0..tex_h).contains(&pos.y)
                    {
                        let idx = (pos.y as u32 * tex_w as u32 + pos.x as u32) as usize;
                        buf[idx] = 255;
                    }
                }

                prev.0 = current_live;
            }
        }
    }
}
