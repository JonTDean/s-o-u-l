//! World2D Bevy resource wrapping the chosen grid backend.

use bevy::prelude::*;
use super::{dim::Dim, Dim2, CellState};
use crate::intelligence_engine::grid::GridBackend;

#[derive(Resource)]
pub struct World2D {
    pub backend: GridBackend,
    pub cell_size: f32,
    /// Background colour for rendering the grid.
    pub bg_color: Color,
}

impl World2D {
    /// Returns neighbour states in the fixed order of `Dim2::NEIGHBOUR_OFFSETS`.
    pub fn neighbourhood(&self, coord: IVec2) -> [CellState; 8] {
        let mut n = [CellState::Dead; 8];
        for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
            n[i] = match &self.backend {
                GridBackend::Dense(g)  => g.get(*off + coord).map_or(CellState::Dead, |c| c.state),
                GridBackend::Sparse(g) => g.get(*off + coord).map_or(CellState::Dead, |c| c.state),
            };
        }
        n
    }
}