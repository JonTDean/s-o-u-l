use super::parallel::{step_sparse_bounded_parallel, step_sparse_dyn_parallel};
use super::*;
use crate::stepper::dense::{parallel::step_dense_dyn_parallel, step_dense};
use crate::{
    core::cell::{CellCtx, CellOutcome, CellState},
    grid::DenseGrid,
};
use glam::IVec3;
use serde_json::{Value, json};
struct Rule(bool);
impl AutomatonRule for Rule {
    type D = Dim;
    fn next_state(&self, c: CellCtx<Dim>, _: &Value) -> CellOutcome {
        let n = c
            .neighbourhood
            .iter()
            .filter(|s| **s != CellState::Dead)
            .count();
        let alive = if self.0 {
            n == 0
        } else {
            n == 3 || (n == 2 && c.self_state != CellState::Dead)
        };
        CellOutcome::Next {
            state: if alive {
                CellState::Alive(255)
            } else {
                CellState::Dead
            },
            memory: c.memory.clone(),
        }
    }
}
#[test]
fn finite_serial_parallel_dense_sparse_parity() {
    let size = UVec3::new(7, 7, 1);
    for spontaneous in [false, true] {
        let mut dense = DenseGrid::blank(size);
        let mut sparse = SparseGrid::default();
        for p in [
            IVec3::new(0, 0, 0),
            IVec3::new(1, 0, 0),
            IVec3::new(2, 0, 0),
        ] {
            dense.set_state(p, CellState::Alive(255));
            sparse.set_state(p, CellState::Alive(255));
        }
        dense.voxels[20].memory = json!({"retained": 1});
        sparse
            .map
            .insert(IVec3::new(6, 2, 0), dense.voxels[20].clone());
        let mut dp = dense.clone();
        let mut sp = sparse.clone();
        for _ in 0..12 {
            step_dense(&mut dense, &Rule(spontaneous), &Value::Null);
            step_dense_dyn_parallel(&mut dp, &Rule(spontaneous), &Value::Null);
            step_sparse_bounded(&mut sparse, &Rule(spontaneous), &Value::Null, size);
            step_sparse_bounded_parallel(&mut sp, &Rule(spontaneous), &Value::Null, size);
            for y in 0..7 {
                for x in 0..7 {
                    let p = IVec3::new(x, y, 0);
                    let expected = &dense.voxels[dense.idx(p)];
                    for actual in [
                        dp.voxels[dp.idx(p)].clone(),
                        sparse.get(p).cloned().unwrap_or_default(),
                        sp.get(p).cloned().unwrap_or_default(),
                    ] {
                        assert_eq!(actual.state, expected.state);
                        assert_eq!(actual.memory, expected.memory);
                    }
                }
            }
            assert!(
                sparse
                    .map
                    .values()
                    .all(|c| c.state != CellState::Dead || !c.memory.is_null())
            );
        }
    }
}
#[test]
fn unbounded_births_and_coordinate_extremes() {
    let mut grid = SparseGrid::default();
    for x in -1..=1 {
        grid.set_state(IVec3::new(x, 0, 0), CellState::Alive(255));
    }
    grid.set_state(IVec3::splat(i32::MAX), CellState::Alive(255));
    let mut parallel = grid.clone();
    step_sparse(&mut grid, &Rule(false), &Value::Null);
    step_sparse_dyn_parallel(&mut parallel, &Rule(false), &Value::Null);
    assert_eq!(
        grid.get(IVec3::new(0, 1, 0)).unwrap().state,
        CellState::Alive(255)
    );
    assert!(grid.get(IVec3::splat(i32::MAX)).is_none());
    assert_eq!(grid.map.len(), parallel.map.len());
    for (p, c) in grid.iter() {
        assert_eq!(parallel.get(p).unwrap().state, c.state);
    }
}
