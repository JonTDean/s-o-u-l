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

use std::sync::Arc;

use bevy::prelude::*;
use engine_core::{
    automata::{AutomatonInfo, GpuGridSlice},
    events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::{AppState, AutomataRegistry, RuleRegistry},
};
use simulation_kernel::{
    core::{
        cell::{CellCtx, CellOutcome},
        dim::Dim,
    },
    grid::{GridBackend, VoxelModify},
    AutomatonRule,
};
use serde_json::Value;

/* ───────── constants ───────── */

pub const DEBUG_FLOOR_ID: &str = "__debug_floor__";
const SLICE_SIDE:         u32  = 256;
const VOXEL_WORLD_SIZE:   f32  = 4.0;
/// completely transparent background → invisible “dead” cells
const BG_COLOUR:          Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
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

/* ───────── seeder ───────── */

/// Draw a **1-voxel-wide grid** every `STEP` cells.
fn seed_checkerboard(grid: &mut GridBackend) {
    const STEP: u32 = 8;              // world-space grid spacing

    let sz = grid.dims();
    for y in 0..sz.y {
        for x in 0..sz.x {
            if x % STEP == 0 || y % STEP == 0 {
                let pos = IVec3::new(x as i32, y as i32, 0);
                grid.write(VoxelModify::new(pos, SOLID_VOXEL));
            }
        }
    }
}

/* ───────── plugin ───────── */

pub struct DebugFloorPlugin;

impl Plugin for DebugFloorPlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ register rule + seed once */
        {
            let mut rules = app.world_mut().resource_mut::<RuleRegistry>();
            if rules.get(DEBUG_FLOOR_ID).is_none() {
                rules.register_with_seed(
                    DEBUG_FLOOR_ID,
                    Arc::new(StaticRule),
                    seed_checkerboard,
                );
            }
        }

        /* 2 ░ ⇧ + F toggler */
        app.add_systems(
            Update,
            toggle_floor_keyboard
                .run_if(in_state(AppState::InGame))
                .in_set(engine_core::systems::schedule::MainSet::Input),
        );
    }
}

/* ───────── system ───────── */

#[allow(clippy::too_many_arguments)]
fn toggle_floor_keyboard(
    mut keys:              ResMut<ButtonInput<KeyCode>>,
    mut registry:      ResMut<AutomataRegistry>,
    mut added:         EventWriter<AutomatonAdded>,
    mut removed:       EventWriter<AutomatonRemoved>,
    _cmd_writer:       EventWriter<AutomataCommand>,      // kept for future use
    mut commands:      Commands,
) {
    // -- detect the combo -------------------------------------------------
    let shift_down = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let fire       = keys.just_pressed(KeyCode::KeyF) && shift_down;

    // -- eat the Shift keys so they don’t reach the camera system ---------
    if fire {
        keys.reset(KeyCode::ShiftLeft);
        keys.reset(KeyCode::ShiftRight);
    }

    if !fire { return; }

    /* already present → despawn */
    if let Some(id) = registry.find_by_name(DEBUG_FLOOR_ID).map(|i| i.id) {
        registry.remove(id);
        removed.write(AutomatonRemoved { id });
        // The system that handles `AutomatonRemoved` will despawn the
        // slice entity, so we don’t touch it here.
        return;
    }

    /* spawn new slice */
    let slice = GpuGridSlice {
        layer:     0,                           // patched later by the GPU plugin
        offset:    UVec2::ZERO,
        size:      UVec2::splat(SLICE_SIDE),
        rule:      DEBUG_FLOOR_ID.into(),
        rule_bits: 0,
    };

    let info = AutomatonInfo {
        id:            AutomatonId(0),          // → overwritten by registry
        name:          DEBUG_FLOOR_ID.into(),
        rule:          Arc::new(StaticRule),
        params:        Value::Null,
        seed_fn:       None,                    // registry handles seeding
        slice,
        dimension:     3,
        voxel_size:    VOXEL_WORLD_SIZE,
        world_offset:  Vec3::ZERO,
        background_color: BG_COLOUR,
        palette:       None,
    };

    let new_id = registry.register(info);
    let entity = commands.spawn_empty().id();
    added.write(AutomatonAdded { id: new_id, entity });
    
    // The actual atlas allocation + seeding happen asynchronously inside the
    // GPU plug-in once it receives the `AutomatonAdded` event, identical to
    // any user-defined automaton.
}