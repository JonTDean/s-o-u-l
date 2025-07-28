//! Seeding helpers for the 1‑D Wolfram elementary CA rules.
//
//  We now support *both* the high‑level `World2D` wrapper (used by
//  CPU‑steppers) **and** the bare‐metal `GridBackend` enum that the
//  scenario spawner uses.  The shared logic lives in the `_backend`
//  helpers; thin wrappers forward the call for convenience.

use bevy::math::IVec2;
use engine_core::{
    core::world::World2D,
    engine::grid::GridBackend,
    core::cell::CellState,
};

/// ───────────────────────────────────────────────────────────────────────
/// Internal helpers (work on a `GridBackend` directly)
/// ───────────────────────────────────────────────────────────────────────

fn seed_middle_band_backend(grid: &mut GridBackend) {
    match grid {
        GridBackend::Dense(g) => {
            // horizontal line at the vertical midpoint
            let y = g.size.y / 2;
            for x in 0..g.size.x {
                let idx = (y * g.size.x + x) as usize;
                g.cells[idx].state = CellState::Alive(255);
            }
        }
        GridBackend::Sparse(s) => {
            // place ~64 cells centred on the origin
            for x in -32..=32 {
                s.set_state(IVec2::new(x, 0), CellState::Alive(255));
            }
        }
    }
}

/// ───────────────────────────────────────────────────────────────────────
/// Public API – **backend** variants (used by the Scenario spawner)
/// ───────────────────────────────────────────────────────────────────────
/// These are the functions registered inside `RuleRegistry`.

pub fn seed_rule30(grid: &mut GridBackend)  { seed_middle_band_backend(grid); }
pub fn seed_rule110(grid: &mut GridBackend) { seed_middle_band_backend(grid); }

/// ───────────────────────────────────────────────────────────────────────
/// Public API – **world** variants (used by the CPU stepper plugin)
/// ───────────────────────────────────────────────────────────────────────

pub fn seed_rule30_world(world: &mut World2D)  { seed_rule30(&mut world.backend); }
pub fn seed_rule110_world(world: &mut World2D) { seed_rule110(&mut world.backend); }
