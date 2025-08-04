use std::sync::Arc;

use bevy::{color::Color, input::ButtonInput, math::{UVec2, Vec3}};
use engine_core::{automata::GpuGridSlice, events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved}, prelude::{AutomataRegistry, AutomatonInfo, Commands, EventWriter, KeyCode, ResMut}};
use serde_json::Value;

use crate::debugging::floor::{StaticRule, DEBUG_FLOOR_ID, SLICE_SIDE, VOXEL_WORLD_SIZE};


#[allow(clippy::too_many_arguments)]
pub(super) fn toggle_floor_keyboard(
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
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.0),  // transparent
        palette: Some(vec![
            Color::srgba(0.0, 0.0, 0.0, 0.0),  // dead cell = transparent
            Color::srgba(0.8, 0.8, 0.8, 0.2),  // alive cell = faint light grey
        ]),
    };

    let new_id = registry.register(info);
    let entity = commands.spawn_empty().id();
    added.write(AutomatonAdded { id: new_id, entity });
    
    // The actual atlas allocation + seeding happen asynchronously inside the
    // GPU plug-in once it receives the `AutomatonAdded` event, identical to
    // any user-defined automaton.
}