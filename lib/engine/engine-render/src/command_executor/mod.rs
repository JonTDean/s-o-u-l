//! engine-render / command_executor / **mod.rs**
//! Spawns automata and converts UI commands → AutomatonInfo.

use std::sync::Arc;

use bevy::prelude::*;
use engine_core::{
    automata::{AutomatonInfo, GpuGridSlice},
    events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::{AppState, AutomataRegistry, RuleRegistry},
};
use serde_json::Value;

use crate::render::camera::systems::WorldCamera;
use crate::WorldGrid;

/* ------------------------------ constants ------------------------- */

const SLICE_SIDE:      u32  = 256;
const WORLD_GRID_SIDE: u32  = 1_024;
const DEFAULT_VOXEL:   f32  = 4.0;
const BG:              Color = Color::srgb(0.07, 0.07, 0.07);

/* ------------------------------ plugin ---------------------------- */

pub struct CommandExecutorPlugin;
impl Plugin for CommandExecutorPlugin {
    fn build(&self, app: &mut App) {
        // we only need logical bounds (and maybe a few debug look-ups) –
        // keep it sparse to avoid 40 GiB allocation balloon.
        app.insert_resource(WorldGrid::new_sparse())
            .add_systems(Update, handle_commands.run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(AppState::MainMenu), purge_on_main_menu);
    }
}

/* ------------------ cleanup when returning to menu ---------------- */

fn purge_on_main_menu(
    mut registry:   ResMut<AutomataRegistry>,
    mut world_grid: ResMut<WorldGrid>,
    mut removed:    EventWriter<AutomatonRemoved>,
) {
    for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
        registry.remove(id);
        removed.write(AutomatonRemoved { id });
    }
    *world_grid = WorldGrid::new_dense(UVec3::splat(WORLD_GRID_SIDE));
}

/* ------------------------ main system ----------------------------- */

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
            /* ---------- spawn ------------------------------------- */
            AutomataCommand::SeedPattern { ref id } => {
                let Some((rule, _seed_fn_opt)) = rules.get(id.as_str()) else {
                    warn!("Unknown automaton pattern “{id}” – ignored");
                    continue;
                };

                /* atlas alloc (2-D inside one layer for now) */
                let Some(slice) = world_grid.allocate(UVec2::splat(SLICE_SIDE)) else {
                    warn!("World grid is full – cannot place new automaton");
                    continue;
                };

                /* GPU-side slice meta */
                let gpu_slice = GpuGridSlice {
                    layer:      0,
                    offset:     UVec2::new(slice.offset.x as u32, slice.offset.y as u32),
                    size:       slice.size.truncate(),
                    rule:       id.clone(),
                    rule_bits:  0b0001_1000, // Conway B3/S23 for now
                };

                /* world-space placement */
                let world_offset = slice.offset.as_vec3() * DEFAULT_VOXEL;

                /* register + ECS entity */
                let info = AutomatonInfo {
                    id:               AutomatonId(0), // overwritten
                    name:             id.clone(),
                    rule:             Arc::clone(rule),
                    params:           Value::Null,
                    seed_fn:          None,
                    slice:            gpu_slice,
                    dimension:        3,
                    voxel_size:       DEFAULT_VOXEL,
                    world_offset,
                    background_color: BG,
                    palette:          None,
                };
                let new_id = registry.register(info);
                let entity = commands.spawn_empty().id();
                added_writer.write(AutomatonAdded { id: new_id, entity });
            }

            /* ---------- clear ------------------------------------- */
            AutomataCommand::Clear => {
                for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
                    registry.remove(id);
                    removed_writer.write(AutomatonRemoved { id });
                }
                *world_grid = WorldGrid::new_dense(UVec3::splat(WORLD_GRID_SIDE));
            }
        }
    }
}

/* ---------------- camera-focus helper ----------------------------- */

pub fn focus_camera_on_new_auto(
    mut added: EventReader<AutomatonAdded>,
    registry:  Res<AutomataRegistry>,
    mut cam_q: Query<&mut Transform, With<WorldCamera>>,
) {
    let Some(ev)   = added.read().last() else { return };
    let Some(info) = registry.get(ev.id)  else { return };
    let Ok(mut tf) = cam_q.single_mut()   else { return };

    let grid_sz = info.slice.size.as_vec2();

    let centre = info.world_offset + grid_sz.extend(0.0) * 0.5 * info.voxel_size;
    tf.translation.x = centre.x;
    tf.translation.y = centre.y;
}
