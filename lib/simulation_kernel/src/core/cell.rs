//! Cell state and serialisable payload.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::core::dim::Dimensionality;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    Dead,
    Alive(u8),           // energy in 1..=complexity
}

impl Default for CellState {
    fn default() -> Self { CellState::Dead }
}

impl From<u8> for CellState {
    #[inline]
    fn from(v: u8) -> Self {
        if v == 0 { Self::Dead } else { Self::Alive(v) }
    }
}

/// Typed-erased per-cell storage.
pub type CellMemory = serde_json::Value;

/// Simulation cell (state + optional JSON memory).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cell {
    pub state:  CellState,
    pub memory: CellMemory,
}

/// Execution context passed to every rule invocation.
pub struct CellCtx<'a, D: Dimensionality> {
    pub self_coord:    D::Coord,
    pub self_state:    CellState,
    pub neighbourhood: &'a [CellState],
    pub memory:        &'a CellMemory,
    pub _marker:       PhantomData<D>,
}

pub enum CellOutcome {
    Unchanged,
    Next { state: CellState, memory: CellMemory },
}

#[inline]
pub fn is_alive(c: &Cell) -> bool {
    !matches!(c.state, CellState::Dead)
}
