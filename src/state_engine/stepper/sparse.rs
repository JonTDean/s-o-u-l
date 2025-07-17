use crate::state_engine::core::{AutomatonRule, CellCtx, CellOutcome, CellState, dim::Dim, Dim2};
use crate::state_engine::grid::SparseGrid;

pub fn step_sparse<R: AutomatonRule<D = Dim2>>(grid: &mut SparseGrid, rule: &R, params: &serde_json::Value) {
    let snapshot = grid.map.clone();
    let mut next = snapshot.clone();

    for (&p, cell) in &snapshot {
        let mut nbhd = [CellState::Dead; 8];
        for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
            nbhd[i] = snapshot.get(&(p + *off)).map_or(CellState::Dead, |c| c.state);
        }
        let ctx = CellCtx { self_coord: p, neighbourhood: &nbhd, memory: &cell.memory };
        if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
            next.entry(p).or_default().state  = state;
            next.entry(p).or_default().memory = memory;
        }
    }

    grid.map = next;
}