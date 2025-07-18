//! Sparse hash‑map grid – for huge worlds with few live cells.
#![allow(clippy::type_complexity)]
use bevy::math::IVec2;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::intelligence_engine::core::{Cell, CellState};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SparseGrid {
    pub map: HashMap<IVec2, Cell>,
}

impl SparseGrid {
    pub fn get(&self, p: IVec2) -> Option<&Cell>       { self.map.get(&p) }
    pub fn get_mut(&mut self, p: IVec2) -> Option<&mut Cell> { self.map.get_mut(&p) }
    pub fn iter(&self) -> impl Iterator<Item = (IVec2, &Cell)> + '_ { self.map.iter().map(|(k,v)| (*k,v)) }

    pub fn set_state(&mut self, p: IVec2, s: CellState) {
        self.map.entry(p).or_default().state = s;
    }
}