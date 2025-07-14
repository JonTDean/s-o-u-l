//! Cell state and serialisable payload.

use serde::{Deserialize, Serialize};

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