//! Parallel steppers for *sparse* grids.
//!
//! This module mirrors `dense_parallel.rs` and provides two helpers:
//!
//! 1. **`step_sparse_parallel`** – generic over a concrete rule type `R`.
//! 2. **`step_sparse_dyn_parallel`** – dynamic‑dispatch version taking a
//!    trait‑object (`&dyn AutomatonRule`).
//!
//! Both functions are *thread‑safe* (require `Sync` on the rule) and keep
//! the mutation phase fully sequential so they can be called from a
//! higher‑level Rayon `par_iter` (or any other executor) without internal
//! data races.

use glam::IVec2;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    core::{
        cell::{Cell, CellCtx, CellOutcome, CellState},
        dim::{Dim, Dim2},
    },
    grid::SparseGrid,
    AutomatonRule,
};

/* ====================================================================== */
/* Generic implementation                                                 */
/* ====================================================================== */

/// **Generic**, compile‑time version (rule is a concrete type `R`).
///
/// The algorithm is split in two passes:
/// 1. Iterate over an *immutable* snapshot of the hash‑map and collect the
///    required updates.
/// 2. Apply those updates sequentially to the live grid.
///
/// Because the first pass only reads shared data it can safely run inside
/// any external parallel iterator.
pub fn step_sparse_parallel<R: AutomatonRule<D = Dim2> + Sync>(
    grid: &mut SparseGrid,
    rule: &R,
    params: &Value,
) {
    let snapshot: HashMap<IVec2, Cell> = grid.map.clone();

    // ── 1. Compute updates ────────────────────────────────────────────
    let mut updates: Vec<(IVec2, CellState, serde_json::Value)> =
        Vec::with_capacity(snapshot.len());

    for (&p, cell) in &snapshot {
        // Build Moore‑8 neighbourhood.
        let mut nbhd = [CellState::Dead; 8];
        for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
            nbhd[i] = snapshot.get(&(p + *off)).map_or(CellState::Dead, |c| c.state);
        }

        let ctx = CellCtx::<Dim2> {
            self_coord: p,
            self_state: cell.state,
            neighbourhood: &nbhd,
            memory: &cell.memory,
            _marker: std::marker::PhantomData,
        };

        if let CellOutcome::Next { state, memory } = rule.next_state(ctx, params) {
            updates.push((p, state, memory));
        }
    }

    // ── 2. Apply updates (sequential → data‑race‑free) ────────────────
    for (p, state, memory) in updates {
        grid.map.entry(p).and_modify(|c| {
            c.state = state;
            c.memory = memory.clone();
        }).or_insert_with(|| Cell { state, memory });
    }
}

/* ====================================================================== */
/* Dynamic‑dispatch implementation                                        */
/* ====================================================================== */

/// Same logic as [`step_sparse_parallel`] but works with a *trait‑object*
/// rule reference (`&dyn AutomatonRule`).  We rebuild the whole `HashMap`
/// at the end to avoid in‑place mutation during the read phase – this is
/// cheap because `HashMap` cloning only copies buckets’ pointers.
pub fn step_sparse_dyn_parallel(
    grid: &mut SparseGrid,
    rule: &(dyn AutomatonRule<D = Dim2> + Sync),
    params: &Value,
) {
    // Read‑only clone (pointer copy per entry).
    let snapshot: HashMap<IVec2, Cell> = grid.map.clone();
    let mut next: HashMap<IVec2, Cell> = snapshot.clone();

    for (&p, cell) in &snapshot {
        // Build Moore‑8 neighbourhood.
        let mut nbhd = [CellState::Dead; 8];
        for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
            nbhd[i] = snapshot.get(&(p + *off)).map_or(CellState::Dead, |c| c.state);
        }

        let ctx = CellCtx::<Dim2> {
            self_coord: p,
            self_state: cell.state,
            neighbourhood: &nbhd,
            memory: &cell.memory,
            _marker: std::marker::PhantomData,
        };

        match rule.next_state(ctx, params) {
            CellOutcome::Next { state, memory } => {
                next.entry(p).or_default().state = state;
                next.entry(p).or_default().memory = memory;
            }
            CellOutcome::Unchanged => { /* leave as‑is */ }
        }
    }

    grid.map = next;
}
