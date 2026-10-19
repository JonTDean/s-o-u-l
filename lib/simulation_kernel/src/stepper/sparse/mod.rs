//! Single-threaded sparse voxel stepper.

pub mod parallel;

mod frontier;
use crate::{AutomatonRule, core::dim::Dim, grid::SparseGrid};
use glam::UVec3;

/// Advance an unbounded quiescent-background rule through its occupied frontier.
/// Rules that create cells with no neighbors require `step_sparse_bounded`.
pub fn step_sparse<R: AutomatonRule<D = Dim> + ?Sized>(
    grid: &mut SparseGrid,
    rule: &R,
    params: &serde_json::Value,
) {
    update(grid, rule, params, None);
}

/// Finite dead-boundary update, including spontaneous background births.
pub fn step_sparse_bounded<R: AutomatonRule<D = Dim> + ?Sized>(
    grid: &mut SparseGrid,
    rule: &R,
    params: &serde_json::Value,
    size: UVec3,
) {
    update(grid, rule, params, Some(size));
}
fn update<R: AutomatonRule<D = Dim> + ?Sized>(
    grid: &mut SparseGrid,
    rule: &R,
    params: &serde_json::Value,
    bounds: Option<UVec3>,
) {
    let next = frontier::candidates(grid, bounds)
        .into_iter()
        .filter_map(|p| frontier::evaluate(grid, p, rule, params, bounds).map(|cell| (p, cell)))
        .collect();
    grid.map = next;
}
