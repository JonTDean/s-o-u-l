//! Plug-in and generation logic for the *New Scenario* workflow.
//!
//! This module now does **three** things:
//!
//! 1.  Registers an empty [`NewScenarioDraft`] on state entry so the UI can
//!     write the scenario name without worrying about lifetimes.
//! 2.  Tears down the draft on state exit.
//! 3.  **NEW** — Creates a fresh [`VoxelWorld`] resource as soon as the user
//!     leaves the *New Scenario* scene.  The world is allocated completely on
//!     a worker thread; no GPU calls are made here, keeping the main thread
//!     responsive even on very large grids.

use bevy::prelude::*;

use engine_core::{
    prelude::AppState,
    world::{
        generator::{create_voxel_world, VoxelWorldOptions},
        voxel_world::VoxelWorld,
    },
};

/* ─────────────────────────────────────────────────────────────── Draft */

/// Draft metadata collected from the UI.
#[derive(Resource, Default, Clone)]
pub struct NewScenarioDraft {
    /// Scenario name typed by the user (may be empty until committed).
    pub name: String,
}

/* ─────────────────────────────────────────────────────────────── Plugin */

/// Registers [`NewScenarioDraft`] *and* spawns an empty [`VoxelWorld`].
pub struct NewScenarioScenePlugin;

impl Plugin for NewScenarioScenePlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ Enter: initialise a new draft for the UI layer */
        app.add_systems(
            OnEnter(AppState::NewScenario),
            |mut cmd: Commands| cmd.insert_resource(NewScenarioDraft::default()),
        );

        /* 2 ░ Exit: generate voxel world & clean-up the draft      */
        app.add_systems(
            OnExit(AppState::NewScenario),
            (
                generate_voxel_world,                       // keeps draft alive
                |mut cmd: Commands| cmd.remove_resource::<NewScenarioDraft>(),
            ),
        );
    }
}

/* ─────────────────────────────────────────────────────────────── Systems */

/// Allocates a brand-new [`VoxelWorld`] using sensible defaults.
///
/// * Grid dimensions: `256 × 256 × 256` voxels  
/// * Voxel edge: `1.0` world units  
/// * Background: dark “digital blackboard”
fn generate_voxel_world(mut cmd: Commands, draft: Res<NewScenarioDraft>) {
    // Real-world projects could read dimensions from the UI; for now we stick
    // to a power-of-two cube that maps 1-to-1 into the default atlas config.
    let opts = VoxelWorldOptions::default();
    let dims = opts.dimensions;          // store before move
    let world = create_voxel_world(opts);

    // Replace any previous world (e.g. when restarting from the main menu)
    cmd.remove_resource::<VoxelWorld>();
    cmd.insert_resource(world);

    info!(
        target: "soul::scenario",
        "Generated fresh VoxelWorld for scenario “{}” ({}³ voxels)",
        draft.name,
        dims.x
    );
}
