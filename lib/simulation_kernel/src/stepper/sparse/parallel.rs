//! Rayon sparse updates share exactly the serial candidate and boundary contract.
use super::frontier;
use crate::{AutomatonRule, core::dim::Dim, grid::SparseGrid};
use glam::UVec3;
use rayon::prelude::*;
use serde_json::Value;

pub fn step_sparse_dyn_parallel(
    grid: &mut SparseGrid,
    rule: &(dyn AutomatonRule<D = Dim> + Sync),
    params: &Value,
) {
    update(grid, rule, params, None);
}

pub fn step_sparse_bounded_parallel(
    grid: &mut SparseGrid,
    rule: &(dyn AutomatonRule<D = Dim> + Sync),
    params: &Value,
    size: UVec3,
) {
    update(grid, rule, params, Some(size));
}
fn update(
    grid: &mut SparseGrid,
    rule: &(dyn AutomatonRule<D = Dim> + Sync),
    params: &Value,
    bounds: Option<UVec3>,
) {
    grid.map = frontier::candidates(grid, bounds)
        .into_par_iter()
        .filter_map(|p| frontier::evaluate(grid, p, rule, params, bounds).map(|cell| (p, cell)))
        .collect();
}
