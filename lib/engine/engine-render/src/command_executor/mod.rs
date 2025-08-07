//! Spawns automata & maps UI commands > AutomatonInfo.

//! Executes high-level automata commands and related helpers.

use bevy::prelude::*;
use engine_common::controls::camera::WorldCamera;
use engine_core::{
    automata::{AutomatonInfo, GpuGridSlice},
    events::{
        AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved
    },
    prelude::{AppState, AutomataRegistry, RuleRegistry},
};
use serde_json::Value;
use std::sync::Arc;

/* ───────────────────────────── constants ───────────────────────────── */

const SLICE_SIDE: u32 = 256;
const DEFAULT_VOXEL: f32 = 4.0;
const BG: Color = Color::srgb(0.07, 0.07, 0.07);

/* ───────────────────────────── plugin ─────────────────────────────── */
/// Routes `AutomataCommand` events to concrete actions.
pub struct CommandExecutorPlugin;
impl Plugin for CommandExecutorPlugin {
    fn build(&self, app: &mut App) {
        use engine_core::systems::schedule::MainSet;

        app
            // existing command router
            .add_systems(
                Update,
                handle_commands
                    .in_set(MainSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
               Update,
               focus_camera_on_new_auto
                   .after(handle_commands)
                   .in_set(MainSet::Input)
                   .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::MainMenu), clear_on_main_menu);
    }
}

fn clear_on_main_menu(
    mut registry: ResMut<AutomataRegistry>,
    mut removed: EventWriter<AutomatonRemoved>,
) {
    for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
        registry.remove(id);
        removed.write(AutomatonRemoved { id });
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_commands(
    mut cmd_reader: EventReader<AutomataCommand>,
    mut registry: ResMut<AutomataRegistry>,
    rules: Res<RuleRegistry>,
    mut added_writer: EventWriter<AutomatonAdded>,
    mut removed_writer: EventWriter<AutomatonRemoved>,
    mut commands: Commands,
) {
    for event in cmd_reader.read() {
        match *event {
            /* ─── spawn default pattern ─────────────────────────── */
            AutomataCommand::SeedPattern { ref id } => {
                let Some((rule, _)) = rules.get(id.as_str()) else {
                    warn!("Unknown automaton pattern “{id}” – ignored");
                    continue;
                };

                /* logical slice – offset always (0,0); layer filled in later */
                let gpu_slice = GpuGridSlice {
                    layer:  0,                       // real Z picked by atlas allocator
                    offset: UVec2::ZERO,
                    size:   UVec2::splat(SLICE_SIDE),
                    depth:  1,                       // every fresh slice starts 1-deep
                    rule:   id.clone(),              // rule id ("lenia", "life", …)
                    rule_bits: 0,
                };

                let info = AutomatonInfo {
                    id: AutomatonId(0), // overwritten by registry
                    name: id.clone(),
                    rule: Arc::clone(rule),
                    params: Value::Null,
                    slice: gpu_slice,
                    voxel_size: DEFAULT_VOXEL,
                    world_offset: Vec3::ZERO,
                    background_color: BG,
                    palette: None,
                };
                let new_id = registry.register(info);

                /* create empty entity – GPU plugin will patch slice & layer */
                let entity = commands.spawn_empty().id();
                added_writer.write(AutomatonAdded { id: new_id, entity });
            }

            /* ─── clear all ──────────────────────────────────────── */
            AutomataCommand::Clear => {
                for id in registry.list().iter().map(|a| a.id).collect::<Vec<_>>() {
                    registry.remove(id);
                    removed_writer.write(AutomatonRemoved { id });
                }
            }
        }
    }
}

/* ─────────────────────── camera helper ───────────────────────────── */
/// Recentre the world camera on the most recently spawned automaton.
pub fn focus_camera_on_new_auto(
    mut added: EventReader<AutomatonAdded>,
    registry: Res<AutomataRegistry>,
    mut cam_q: Query<&mut Transform, With<WorldCamera>>,
) {
    let Some(ev) = added.read().last() else {
        return;
    };
    let Some(info) = registry.get(ev.id) else {
        return;
    };
    let Ok(mut tf) = cam_q.single_mut() else {
        return;
    };

    let grid_sz = info.slice.size.as_vec2();
    let centre = info.world_offset + grid_sz.extend(0.0) * 0.5 * info.voxel_size;

    tf.translation.x = centre.x;
    tf.translation.y = centre.y;
}

