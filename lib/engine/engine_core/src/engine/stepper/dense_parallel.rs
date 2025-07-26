use bevy::math::IVec2;
use serde_json::Value;

use crate::{core::{AutomatonRule, CellCtx, CellOutcome, CellState, Dim, Dim2}, engine::grid::DenseGrid};

pub fn step_dense_parallel<R: AutomatonRule<D = Dim2> + Sync>(
    grid: &mut DenseGrid,
    rule: &R,
    params: &Value,
) {
    let snapshot = grid.cells.clone();
    let size = grid.size;
    let updates: Vec<(usize, CellState)> = (0..snapshot.len())
        .into_iter()
        .filter_map(|idx| {
            let x = (idx as u32 % size.x) as i32;
            let y = (idx as u32 / size.x) as i32;
            let p = IVec2::new(x, y);

            // Build Moore neighbourhood.
            let mut nbhd = [CellState::Dead; 8];
            for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
                let q = p + *off;
                if (0..size.x as i32).contains(&q.x) && (0..size.y as i32).contains(&q.y) {
                    nbhd[i] = snapshot[grid.idx(q)].state;
                }
            }

            let ctx = CellCtx {
                self_coord: p,
                self_state: snapshot[idx].state,
                neighbourhood: &nbhd,
                memory: &snapshot[idx].memory,
                _marker: std::marker::PhantomData::<Dim2>,
            };

            match rule.next_state(ctx, params) {
                CellOutcome::Next { state, .. } => Some((idx, state)),
                _ => None,
            }
        })
        .collect();

    // Apply updates sequentially (unordered, data‑race‑free).
    for (idx, state) in updates {
        grid.cells[idx].state = state;
    }
}
