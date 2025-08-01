//! Simple swarm-analysis helpers (now 3-D).

use glam::IVec3;
use simulation_kernel::{
    core::cell::is_alive,
    grid::{DenseGrid, GridBackend},
};
use std::collections::HashSet;

/// Return the set of coordinates that are *alive* in a dense grid.
pub fn live_cells_dense(grid: &DenseGrid) -> HashSet<IVec3> {
    grid.voxels
        .iter()
        .enumerate()
        .filter_map(|(idx, cell)| is_alive(cell).then(|| {
            let x =  idx as u32              % grid.size.x;
            let y = (idx as u32 / grid.size.x)    % grid.size.y;
            let z =  idx as u32 / (grid.size.x * grid.size.y);
            IVec3::new(x as i32, y as i32, z as i32)
        }))
        .collect()
}

/// Works on either dense or sparse back-ends.
pub fn live_cell_set(grid: &GridBackend) -> HashSet<IVec3> {
    match grid {
        GridBackend::Dense(g)  => live_cells_dense(g),
        GridBackend::Sparse(s) => s.iter()
                                   .filter(|(_, cell)| is_alive(cell))
                                   .map(|(p, _)| p)
                                   .collect(),
    }
}

/// Percentage of active voxels inside the grid’s AABB.
pub fn activity_ratio(grid: &GridBackend) -> f32 {
    let live = live_cell_set(grid).len() as f32;
    let total = match grid {
        GridBackend::Dense(g)  => (g.size.x * g.size.y * g.size.z) as f32,
        GridBackend::Sparse(s) => s.map.len() as f32,            // only tracked cells
    };
    if total == 0.0 { 0.0 } else { live / total }
}
