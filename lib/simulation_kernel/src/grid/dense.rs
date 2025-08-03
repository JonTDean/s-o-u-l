//! **Dense 3-D voxel grid** (Z-major layout).

use glam::{IVec3, UVec3};
use serde::{Deserialize, Serialize};

use crate::core::cell::{Cell, CellState};

#[derive(Clone, Serialize, Deserialize)]
pub struct DenseGrid {
    pub voxels: Vec<Cell>,
    pub size:   UVec3,          // (x, y, z)
}

impl DenseGrid {
    #[inline]
    pub fn idx(&self, p: IVec3) -> usize {
        ((p.z as u32 * self.size.y + p.y as u32) * self.size.x + p.x as u32) as usize
    }

    #[inline]
    pub fn get(&self, p: IVec3) -> Option<&Cell> {
        if (0..self.size.x as i32).contains(&p.x)
            && (0..self.size.y as i32).contains(&p.y)
            && (0..self.size.z as i32).contains(&p.z)
        {
            self.voxels.get(self.idx(p))
        } else { None }
    }

    #[inline]
    pub fn get_mut(&mut self, p: IVec3) -> Option<&mut Cell> {
        if (0..self.size.x as i32).contains(&p.x)
            && (0..self.size.y as i32).contains(&p.y)
            && (0..self.size.z as i32).contains(&p.z)
        {
            let idx = self.idx(p);
            self.voxels.get_mut(idx)
        } else { None }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (IVec3, &Cell)> + '_ {
        let size = self.size;
        self.voxels.iter().enumerate().map(move |(i, c)| {
            let x =  i as u32              % size.x;
            let y = (i as u32 / size.x)    % size.y;
            let z =  i as u32 / (size.x * size.y);
            (IVec3::new(x as i32, y as i32, z as i32), c)
        })
    }

    #[inline]
    pub fn blank(size: UVec3) -> Self {
        Self {
            voxels: vec![Cell::default(); (size.x * size.y * size.z) as usize],
            size,
        }
    }

    #[inline]
    pub fn set_state(&mut self, p: IVec3, s: CellState) {
        if let Some(c) = self.get_mut(p) { c.state = s; }
    }
}
