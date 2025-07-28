//! Converts UI‑level `AutomataCommand`s into concrete `AutomatonInfo`s
//! and notifies both the CPU renderer *and* the GPU slice allocator.

use std::sync::Arc;

use bevy::prelude::*;
use engine_core::{
    engine::{
        grid::{DenseGrid, GridBackend},
        worldgrid::WorldGrid,
    },
    events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved}, state::AppState,
};
use serde_json::Value;

use crate::registry::{AutomataRegistry, AutomatonInfo, RuleRegistry};

/* ───────────────────────── Constants ─────────────────────────────── */

/// Side length of **one** automaton slice (kept at 256 px for now).
const SLICE_SIDE:      u32  = 256;
/// Side length of the global tiled atlas; plenty of room for > 10 slices.
const WORLD_GRID_SIDE: u32  = 1_024;
/// Default visual cell size in world‑space units.
const DEFAULT_CELL:    f32  = 4.0;
/// Default background colour.
const BG:              Color = Color::srgb(0.07, 0.07, 0.07);

/* ────────────────────────── Plugin ───────────────────────────────── */

pub struct CommandExecutorPlugin;
impl Plugin for CommandExecutorPlugin {
    fn build(&self, app: &mut App) {
        // Provide one blank dense world‑grid up‑front (room for many slices).
        app.insert_resource(WorldGrid::new_dense(UVec2::splat(WORLD_GRID_SIDE)))
            .add_systems(
                Update,
                handle_commands
                    .run_if(in_state(engine_core::state::AppState::InGame)),
            );
            
     // Main‑menu reset – one‑shot, runs *after* we quit InGame.
        app.add_systems(
            OnEnter(AppState::MainMenu),
            purge_on_main_menu,
        );
    }
}

/* ───────── new system ───────── */
/// Despawns every automaton & resets the global dense world‑grid.
fn purge_on_main_menu(
    mut registry:       ResMut<AutomataRegistry>,
    mut world_grid:     ResMut<WorldGrid>,
    mut removed:        EventWriter<AutomatonRemoved>,
) {
    // Emit `AutomatonRemoved` for every live automaton so the render side
    // can safely despawn its quads / textures.
    for id in registry
        .list()
        .iter()
        .map(|a| a.id)
        .collect::<Vec<_>>()                         // avoid borrow issues
    {
        registry.remove(id);
        removed.write(AutomatonRemoved { id });
    }

    // Give the allocator a brand‑new blank 1 024 × 1 024 dense grid.
    *world_grid = WorldGrid::new_dense(UVec2::splat(WORLD_GRID_SIDE));
}

/* ───────────────────────── System ────────────────────────────────── */

#[allow(clippy::too_many_arguments)]
fn handle_commands(
    mut cmd_reader:     EventReader<AutomataCommand>,
    mut registry:       ResMut<AutomataRegistry>,
    rules:              Res<RuleRegistry>,
    mut world_grid:     ResMut<WorldGrid>,
    mut added_writer:   EventWriter<AutomatonAdded>,
    mut removed_writer: EventWriter<AutomatonRemoved>,
    mut commands:       Commands,
) {
    for event in cmd_reader.read() {
        match *event {
            /* ---------- spawn one automaton ------------------------ */
            AutomataCommand::SeedPattern { ref id } => {
                /* rule + default‑seed lookup */
                let Some((rule, seed_fn_opt)) = rules.get(id.as_str()) else {
                    warn!("Unknown automaton pattern “{id}” – ignored");
                    continue;
                };

                /* request a 256 × 256 slice from the tiled atlas */
                let size  = UVec2::splat(SLICE_SIDE);
                if world_grid.allocate(size).is_none() {
                    warn!("World grid is full – cannot place new automaton");
                    continue;
                }

                /* build an *isolated* scratch grid and run the seeder */
                let mut slice_backend = GridBackend::Dense(DenseGrid::blank(size));
                if let Some(seed) = seed_fn_opt {
                    seed(&mut slice_backend);
                }

                /* register automaton and spawn its ECS entity */
                let info = AutomatonInfo {
                    id:               AutomatonId(0), // overwritten by `register`
                    name:             id.clone(),
                    rule:             Arc::clone(rule),
                    params:           Value::Null,
                    seed_fn:          None,
                    grid:             slice_backend,  // ← own private grid
                    dimension:        2,
                    cell_size:        DEFAULT_CELL,
                    background_color: BG,
                    palette:          None,
                };
                let new_id = registry.register(info);
                let entity = commands.spawn_empty().id();
                added_writer.write(AutomatonAdded { id: new_id, entity });
            }

            /* ---------- clear all ---------------------------------- */
            AutomataCommand::Clear => {
                /* despawn every automaton */
                for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
                    registry.remove(id);
                    removed_writer.write(AutomatonRemoved { id });
                }
                /* reset atlas allocator */
                *world_grid = WorldGrid::new_dense(UVec2::splat(WORLD_GRID_SIDE));
            }
        }
    }
}
