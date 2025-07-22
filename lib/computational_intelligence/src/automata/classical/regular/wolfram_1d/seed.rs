use bevy::math::IVec2;
use engine_core::core::{World2D, cell::CellState};
use engine_core::engine::grid::GridBackend;

/// Fill the middle row with live cells (works for any 1‑D Wolfram rule).
pub fn seed_middle_band(world: &mut World2D) {
    if let GridBackend::Dense(grid) = &mut world.backend {
        let y = grid.size.y / 2;
        for x in 0..grid.size.x {
            let idx = (y * grid.size.x + x) as usize;
            grid.cells[idx].state = CellState::Alive(255);
        }
    }
}


/// Put **one live cell in the logical centre** of the world.
/// – Dense  → arithmetic centre of the allocated grid  
/// – Sparse → origin `(0, 0)` is “good enough” (sparse worlds are unbounded)
pub fn seed_center_cell(world: &mut World2D) {
    match &mut world.backend {
        GridBackend::Dense(grid) => {
            let cx  = (grid.size.x / 2) as usize;
            let cy  = (grid.size.y / 2) as usize;
            let idx = cy * grid.size.x as usize + cx;
            grid.cells[idx].state = CellState::Alive(255);
        }
        GridBackend::Sparse(sparse) => {
            sparse.set_state(IVec2::ZERO, CellState::Alive(255));
        }
    }
}

/* ─── exported wrappers ─────────────────────────────────────────────── */

pub fn seed_rule30 (w: &mut World2D) { seed_middle_band(w); }
pub fn seed_rule110(w: &mut World2D) { seed_middle_band(w); }