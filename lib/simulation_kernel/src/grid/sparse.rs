//! **Sparse 3-D voxel grid** (`HashMap` backend).

use glam::IVec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::cell::{Cell, CellState};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SparseGrid {
    pub map: HashMap<IVec3, Cell>,
}

impl SparseGrid {
    #[inline] pub fn get    (&self, p: IVec3) -> Option<&Cell>         { self.map.get(&p) }
    #[inline] pub fn get_mut(&mut self, p: IVec3) -> Option<&mut Cell> { self.map.get_mut(&p) }

    #[inline]
    pub fn set_state(&mut self, p: IVec3, s: CellState) {
        self.map.entry(p).or_default().state = s;
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (IVec3, &Cell)> + '_ {
        self.map.iter().map(|(k, v)| (*k, v))
    }
}
