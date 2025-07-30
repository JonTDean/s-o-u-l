pub mod parallel;

use crate::{core::{cell::{CellCtx, CellOutcome, CellState}, dim::{Dim, Dim2}}, grid::SparseGrid, AutomatonRule};


#[inline(always)]
pub fn step_sparse<R: AutomatonRule<D = Dim2>>(grid: &mut SparseGrid, rule: &R, params: &serde_json::Value) {
    let snapshot = grid.map.clone();
    let mut next = snapshot.clone();

    for (&p, cell) in &snapshot {
        let mut nbhd = [CellState::Dead; 8];

        for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
            nbhd[i] = snapshot.get(&(p + *off)).map_or(CellState::Dead, |c| c.state);
        }
        
        let ctx = CellCtx {
            self_coord: p,
            self_state: cell.state,
            neighbourhood: &nbhd,
            memory: &cell.memory,
            _marker: std::marker::PhantomData::<Dim2>,
        };

        if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
            next.entry(p).or_default().state  = state;
            next.entry(p).or_default().memory = memory;
        }
    }

    grid.map = next;
}