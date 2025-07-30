//! Helpers for translating between `engine_core::core::Cell`
//! and higher‑level CI analysis code.
//
//! Everything here is a placeholder; extend as needed.

use simulation_kernel::core::cell::{Cell, CellState};


#[inline]
pub fn is_alive(c: &Cell) -> bool {
    !matches!(c.state, CellState::Dead)
}
