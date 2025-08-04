//! tooling/debugging/floor.rs
//! ---------------------------------------------------------------------------
//! **Voxel Debug Floor – decoupled from the camera**
//!
//! This module removes the old “floor grid” from the `Gizmos` camera overlay
//! and re-implements it as a *real* automaton-like slice in the global 3-D
//! voxel atlas.  The new floor therefore:
//!
//! • **Never scales** when you pan / zoom the camera (it lives in world space
//!   like any other automaton slice).  
//! • Can be toggled on/off at run-time with **⇧ + F** *(Shift + F)* or via
//!   a scripted `AutomataCommand::SeedPattern { id = "__debug_floor__" }`.  
//! • Uses the exact same GPU pipeline (compute + mesh shader / dual contour)
//!   as every other voxel structure, so performance and appearance stay
//!   consistent.
//!
//! ## How it works
//!
//! 1. A tiny **static rule** (`StaticRule`) is registered once with the global
//!    [`RuleRegistry`]. It never mutates voxels – we only use it to hitch a
//!    ride on the usual automaton infrastructure (atlas allocation, GPU
//!    pathways, etc.).
//! 2. The companion `seed_checkerboard` function fills a 256 × 256 × 1 slice
//!    with a simple 2-colour checkerboard (`255` for the dark squares, `0`
//!    elsewhere). This produces a visually distinctive floor that is easy to
//!    differentiate from live simulation voxels.
//! 3. The [`DebugFloorPlugin`] wires everything together:
//!    • Registers the rule + seed,
//!    • Handles the ⇧ + F hot-key to spawn / despawn,
//!    • Re-uses the existing *focus-camera-on-new-automaton* helper so the
//!      view snaps to the floor the first time you create it.
//
//! ### Thread-Safety
//! All data accessed here is either:
//! • read-only (`&Res<_>`),  
//! • mutated behind Bevy’s schedule barriers (`Commands`), or  
//! • `Send + Sync` inside `Arc`.  
//!
//! No global mutable state is accessed concurrently, so the code is fully
//! multi-thread-safe under Bevy’s task scheduler.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//!
//! tooling/debugging/floor.rs  –  immutable “checkerboard” debug slice
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

use simulation_kernel::{
    core::{
        cell::{CellCtx, CellOutcome},
        dim::Dim,
    },
    AutomatonRule,
};

pub mod register;
pub mod plugin;
pub mod seeder;
pub mod system;

/* ───────── constants ───────── */

pub const DEBUG_FLOOR_ID: &str = "__debug_floor__";
const SLICE_SIDE:         u32  = 256;
const VOXEL_WORLD_SIZE:   f32  = 1.0;

/// colour for the grid lines (will use the slice’s **alive** colour)
const SOLID_VOXEL:        u8   = 255;

#[derive(Component)]
pub struct NoOriginShift;

/* ───────── dummy rule ───────── */

#[derive(Clone)]
struct StaticRule;

impl AutomatonRule for StaticRule {
    type D = Dim;
    fn next_state<'a>(
        &self,
        _ctx:    CellCtx<'a, Self::D>,
        _params: &serde_json::Value,
    ) -> CellOutcome {
        CellOutcome::Unchanged
    }
}
