pub mod parallel;

use glam::IVec2;
use serde_json::Value;

use crate::{core::{cell::{CellCtx, CellOutcome, CellState}, dim::{Dim, Dim2}}, grid::DenseGrid, AutomatonRule};


#[inline(always)]
pub fn step_dense<R: AutomatonRule<D = Dim2>>(
    grid:   &mut DenseGrid,
    rule:   &R,
    params: &Value,
) {
    let snapshot = grid.cells.clone();
    let mut next = snapshot.clone();

    for y in 0..grid.size.y as i32 {
        for x in 0..grid.size.x as i32 {
            let p   = IVec2::new(x, y);
            let idx = grid.idx(p);

            /* build Moore‑8 neighbourhood */
            let mut nbhd = [CellState::Dead; 8];
            for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
                let q = p + *off;
                if (0..grid.size.x as i32).contains(&q.x)
                    && (0..grid.size.y as i32).contains(&q.y)
                {
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

            if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
                next[idx].state  = state;
                next[idx].memory = memory;
            }
        }
    }

    grid.cells = next;
}
