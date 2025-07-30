use glam::{IVec2, UVec2};
use serde::{Serialize, Deserialize};

use crate::core::cell::Cell;


#[derive(Clone, Serialize, Deserialize)]
pub struct DenseGrid {
    pub cells: Vec<Cell>,
    pub size:  UVec2,
}

impl DenseGrid {
    #[inline]
    pub fn idx(&self, p: IVec2) -> usize {
        (p.y as u32 * self.size.x + p.x as u32) as usize
    }

    pub fn get(&self, p: IVec2) -> Option<&Cell> {
        if (0..self.size.x as i32).contains(&p.x) && (0..self.size.y as i32).contains(&p.y) {
            self.cells.get(self.idx(p))
        } else { None }
    }

    pub fn get_mut(&mut self, p: IVec2) -> Option<&mut Cell> {
        if (0..self.size.x as i32).contains(&p.x) && (0..self.size.y as i32).contains(&p.y) {
            // compute the index directly (no extra immutable borrow)
            let idx = (p.y as u32 * self.size.x + p.x as u32) as usize;
            self.cells.get_mut(idx)
        } else { None }
    }


    pub fn iter(&self) -> impl Iterator<Item = (IVec2, &Cell)> + '_ {
        let size = self.size;
        self.cells.iter().enumerate().map(move |(i, c)| {
            let x = (i as u32 % size.x) as i32;
            let y = (i as u32 / size.x) as i32;
            (IVec2::new(x, y), c)
        })
    }

    pub fn blank(size: UVec2) -> Self {
        Self { cells: vec![Cell::default(); (size.x * size.y) as usize], size }
    }
}