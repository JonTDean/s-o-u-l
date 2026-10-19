//! Sparse candidates include absent neighbors; meaningful dead-cell memory survives.
use crate::{
    AutomatonRule,
    core::{
        cell::{Cell, CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::SparseGrid,
};
use glam::{IVec3, UVec3};
use std::collections::HashSet;

pub(super) fn candidates(grid: &SparseGrid, bounds: Option<UVec3>) -> Vec<IVec3> {
    if let Some(size) = bounds {
        // A finite domain also supports rules that create cells from an empty background.
        return (0..size.z)
            .flat_map(|z| {
                (0..size.y).flat_map(move |y| {
                    (0..size.x).map(move |x| IVec3::new(x as i32, y as i32, z as i32))
                })
            })
            .collect();
    }
    let mut positions = HashSet::new();
    for &p in grid.map.keys() {
        positions.insert(p);
        for offset in Dim::NEIGHBOUR_OFFSETS {
            if let (Some(x), Some(y), Some(z)) = (
                p.x.checked_add(offset.x),
                p.y.checked_add(offset.y),
                p.z.checked_add(offset.z),
            ) {
                positions.insert(IVec3::new(x, y, z));
            }
        }
    }
    positions.into_iter().collect()
}

pub(super) fn evaluate(
    grid: &SparseGrid,
    p: IVec3,
    rule: &(impl AutomatonRule<D = Dim> + ?Sized),
    params: &serde_json::Value,
    bounds: Option<UVec3>,
) -> Option<Cell> {
    let blank = Cell::default();
    let cell = grid.map.get(&p).unwrap_or(&blank);
    let mut neighbors = [CellState::Dead; 26];
    for (index, offset) in Dim::NEIGHBOUR_OFFSETS.iter().enumerate() {
        let Some(x) = p.x.checked_add(offset.x) else {
            continue;
        };
        let Some(y) = p.y.checked_add(offset.y) else {
            continue;
        };
        let Some(z) = p.z.checked_add(offset.z) else {
            continue;
        };
        let q = IVec3::new(x, y, z);
        if bounds.is_none_or(|size| q.cmpge(IVec3::ZERO).all() && q.cmplt(size.as_ivec3()).all()) {
            neighbors[index] = grid.map.get(&q).map_or(CellState::Dead, |c| c.state);
        }
    }
    let context = CellCtx::<Dim> {
        self_coord: p,
        self_state: cell.state,
        neighbourhood: &neighbors,
        memory: &cell.memory,
        _marker: std::marker::PhantomData,
    };
    let next = match rule.next_state(context, params) {
        CellOutcome::Unchanged => cell.clone(),
        CellOutcome::Next { state, memory } => Cell { state, memory },
    };
    (next.state != CellState::Dead || !next.memory.is_null()).then_some(next)
}
