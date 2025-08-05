//! Spawns automata & maps UI commands > AutomatonInfo.

//! Executes high-level automata commands and related helpers.

use bevy::prelude::*;
use engine_common::controls::camera::WorldCamera;
use engine_core::{
    automata::{AutomatonInfo, GpuGridSlice},
    events::{
        AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved, GenerateDebugFloor,
        ToggleDebugGrid,
    },
    prelude::{AppState, AutomataRegistry, RuleRegistry},
};
use serde_json::Value;
use std::sync::Arc;

use crate::render::materials::debug::debug_grid::DebugGridTag;
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
            // NEW: floor & grid handlers (same set so they run before Logic/Render)
            .add_systems(
                Update,
                (handle_generate_floor, handle_toggle_grid)
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
                    layer: 0, // real Z picked by atlas allocator
                    offset: UVec2::ZERO,
                    size: UVec2::splat(SLICE_SIDE),
                    rule: id.clone(),
                    rule_bits: 0,
                };

                let info = AutomatonInfo {
                    id: AutomatonId(0), // overwritten by registry
                    name: id.clone(),
                    rule: Arc::clone(rule),
                    params: Value::Null,
                    seed_fn: None,
                    slice: gpu_slice,
                    dimension: 3,
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

fn handle_generate_floor(
    mut ev: EventReader<GenerateDebugFloor>,
    mut out: EventWriter<AutomataCommand>,
) {
    if !ev.is_empty() {
        out.write(AutomataCommand::SeedPattern {
            id: "__debug_floor__".into(),
        });
        ev.clear();
    }
}

fn handle_toggle_grid(
    mut ev: EventReader<ToggleDebugGrid>,
    mut q: Query<&mut Visibility, With<DebugGridTag>>,
) {
    if ev.is_empty() {
        return;
    }
    if let Ok(mut vis) = q.single_mut() {
        // <- unwrap the Result
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Visible => Visibility::Hidden,
            Visibility::Inherited => Visibility::Hidden,
        };
    }
    ev.clear();
}
