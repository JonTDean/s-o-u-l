//! Cell state and serialisable payload.

use std::marker::PhantomData;

use bevy::math::IVec2;
use serde::{Deserialize, Serialize};

use crate::core::Dim;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    Dead,
    /// Positive energy value (1 ..= complexity_level).
    Alive(u8),
}

impl Default for CellState {
    fn default() -> Self { CellState::Dead }
}

/// Optional, typed‑erased per‑cell storage.
pub type CellMemory = serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cell {
    pub state:  CellState,
    pub memory: CellMemory,
}

/// Context provided to every rule invocation.
pub struct CellCtx<'a, D: Dim> {
    pub self_coord:   IVec2,
    pub self_state:   CellState,
    pub neighbourhood: &'a [CellState; 8],
    pub memory:       &'a CellMemory,        // <- align with `cell.rs`
    pub _marker:          PhantomData<D>,        // <- silences the lint ✔
}

pub enum CellOutcome {
    Unchanged,
    Next { state: CellState, memory: CellMemory },
}
