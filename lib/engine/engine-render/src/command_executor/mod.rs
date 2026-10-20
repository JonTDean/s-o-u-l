//! Spawns automata & maps UI commands > AutomatonInfo.

//! Executes high-level automata commands and related helpers.

use bevy::prelude::*;
use engine_common::controls::camera::WorldCamera;
use engine_core::{
    automata::{
        AutomatonInfo, GpuGridSlice,
        channel::{ChannelId, ChannelIds, ChannelSet, Channels},
        lenia::LeniaSimulation,
    },
    events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::{AppState, AutomataRegistry, RuleRegistry},
};
use serde_json::Value;
use std::sync::Arc;
pub mod channels;
mod restore;

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
        engine_core::automata::channel::ensure_plugin(app);
        app.add_systems(
            Update,
            (
                channels::route.in_set(ChannelSet::Route),
                channels::publish.in_set(ChannelSet::Publish),
            ),
        );

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
                    .in_set(MainSet::Render)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::MainMenu), clear_on_main_menu);
    }
}

fn clear_on_main_menu(
    mut registry: ResMut<AutomataRegistry>,
    mut removed: EventWriter<AutomatonRemoved>,
    owners: Query<&ChannelId, With<LeniaSimulation>>,
    mut ids: ResMut<ChannelIds>,
) {
    for id in &owners {
        ids.release(*id);
    }
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
    mut ids: ResMut<ChannelIds>,
    mut channels: ResMut<Channels>,
    owners: Query<&ChannelId, With<LeniaSimulation>>,
) {
    for event in cmd_reader.read() {
        match *event {
            AutomataCommand::RestoreLenia(ref saved) => {
                let Some(channel) = ids.allocate() else {
                    warn!("8 of 8 open — close one to add another");
                    continue;
                };
                match restore::restore(
                    saved,
                    &mut registry,
                    &rules,
                    &mut commands,
                    &mut added_writer,
                ) {
                    Some(entity) => {
                        channels::adopt(&mut commands.entity(entity), channel, &mut channels)
                    }
                    None => ids.release(channel),
                }
            }
            /* ─── spawn default pattern ─────────────────────────── */
            AutomataCommand::SeedPattern { ref id } => {
                // CPU and symbolic families have their own inspectable sessions.
                if id != "lenia" && id != "lenia:orbium" {
                    continue;
                }
                let Some((rule, _)) = rules.get(id.as_str()) else {
                    warn!("Unknown automaton pattern “{id}” – ignored");
                    continue;
                };
                let Some(channel) = ids.allocate() else {
                    warn!("8 of 8 open — close one to add another");
                    continue;
                };

                /* logical slice – offset always (0,0); layer filled in later */
                let is_lenia = id == "lenia" || id == "lenia:orbium";
                let gpu_slice = GpuGridSlice {
                    layer: 0, // real Z picked by atlas allocator
                    offset: UVec2::ZERO,
                    size: UVec2::splat(if is_lenia { 64 } else { SLICE_SIDE }),
                    depth: if is_lenia { 8 } else { 1 },
                    rule: id.clone(), // rule id ("lenia", "life", …)
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
                let mut owner = commands.spawn(new_id);
                if is_lenia {
                    use simulation_kernel::core::lenia::LeniaSeed;
                    owner.insert(engine_core::automata::lenia::LeniaSimulation {
                        paused: true,
                        seed: if id == "lenia:orbium" {
                            LeniaSeed::Ring
                        } else {
                            LeniaSeed::Blob
                        },
                        ..default()
                    });
                }
                channels::adopt(&mut owner, channel, &mut channels);
                let entity = owner.id();
                added_writer.write(AutomatonAdded { id: new_id, entity });
            }

            /* ─── clear all ──────────────────────────────────────── */
            AutomataCommand::Clear => {
                for id in &owners {
                    ids.release(*id);
                }
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
