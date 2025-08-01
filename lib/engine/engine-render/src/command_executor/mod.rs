//! Converts UI-level `AutomataCommand`s into concrete `AutomatonInfo`s
//! and notifies both the CPU renderer *and* the GPU slice allocator.

use std::sync::Arc;

use bevy::prelude::*;
use engine_core::{
    events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::{AppState, AutomataRegistry, AutomatonInfo, RuleRegistry},
};
use serde_json::Value;
use simulation_kernel::grid::{DenseGrid, GridBackend};

use crate::{render::camera::systems::WorldCamera, WorldGrid};

/* ───────────────────────── Constants ─────────────────────────────── */

const SLICE_SIDE:      u32  = 256;   // side length of one automaton slice
const WORLD_GRID_SIDE: u32  = 1_024; // global atlas side length
const DEFAULT_CELL:    f32  = 4.0;   // default cell size in world units
const BG:              Color = Color::srgb(0.07, 0.07, 0.07);

/* ────────────────────────── Plugin ───────────────────────────────── */

pub struct CommandExecutorPlugin;
impl Plugin for CommandExecutorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGrid::new_dense(UVec2::splat(WORLD_GRID_SIDE)))
            .add_systems(Update, handle_commands.run_if(in_state(AppState::InGame)))
            // one-shot cleanup when leaving the game
            .add_systems(OnEnter(AppState::MainMenu), purge_on_main_menu);
    }
}

/* ───────── one-shot cleanup when returning to the main menu ─────── */

fn purge_on_main_menu(
    mut registry:   ResMut<AutomataRegistry>,
    mut world_grid: ResMut<WorldGrid>,
    mut removed:    EventWriter<AutomatonRemoved>,
) {
    for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
        registry.remove(id);
        removed.write(AutomatonRemoved { id });
    }
    *world_grid = WorldGrid::new_dense(UVec2::splat(WORLD_GRID_SIDE));
}

/* ───────────────────────── System ───────────────────────────────── */

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
                let Some((rule, seed_fn_opt)) = rules.get(id.as_str()) else {
                    warn!("Unknown automaton pattern “{id}” – ignored");
                    continue;
                };

                let size = UVec2::splat(SLICE_SIDE);
                let Some(slice) = world_grid.allocate(size) else {
                    warn!("World grid is full – cannot place new automaton");
                    continue;
                };

                // build an isolated scratch grid and run the seeder
                let mut slice_backend = GridBackend::Dense(DenseGrid::blank(size));
                if let Some(seed) = seed_fn_opt {
                    seed(&mut slice_backend);
                }

                // convert slice offset (cells) → world-space units
                let world_offset = slice.offset.as_vec2() * DEFAULT_CELL;

                // register automaton and spawn its ECS entity
                let info = AutomatonInfo {
                    id:               AutomatonId(0), // overwritten below
                    name:             id.clone(),
                    rule:             Arc::clone(rule),
                    params:           Value::Null,
                    seed_fn:          None,
                    grid:             slice_backend,
                    dimension:        2,
                    cell_size:        DEFAULT_CELL,
                    background_color: BG,
                    palette:          None,
                    world_offset,
                };
                let new_id = registry.register(info);
                let entity = commands.spawn_empty().id();
                added_writer.write(AutomatonAdded { id: new_id, entity });
            }

            /* ---------- clear all ---------------------------------- */
            AutomataCommand::Clear => {
                for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
                    registry.remove(id);
                    removed_writer.write(AutomatonRemoved { id });
                }
                *world_grid = WorldGrid::new_dense(UVec2::splat(WORLD_GRID_SIDE));
            }
        }
    }
}

/* ────────────────────── helper: get grid size ───────────────────── */

#[inline]
fn grid_texel_size(grid: &GridBackend) -> UVec2 {
    match grid {
        GridBackend::Dense(g)  => g.size,
        GridBackend::Sparse(_) => UVec2::new(512, 512),
    }
}

/* ─────────────────────── camera-focus helper ────────────────────── */

pub fn focus_camera_on_new_auto(
    mut added: EventReader<AutomatonAdded>,
    registry:  Res<AutomataRegistry>,
    mut cam_q: Query<&mut Transform, With<WorldCamera>>,
) {
    let Some(ev)   = added.read().last() else { return };
    let Some(info) = registry.get(ev.id)  else { return };
    let Ok(mut tf) = cam_q.single_mut()   else { return };

    let grid_sz = grid_texel_size(&info.grid);

    // world_offset already includes cell_size scaling
    let slice_centre = info.world_offset + grid_sz.as_vec2() * 0.5 * info.cell_size;

    tf.translation.x = slice_centre.x;
    tf.translation.y = slice_centre.y;
}
