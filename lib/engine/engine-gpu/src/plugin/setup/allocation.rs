//! Allocate GPU slices for newly registered automata.
use crate::plugin::atlas::AtlasAllocator;
use bevy::prelude::*;
use engine_core::{
    automata::GpuGridSlice,
    events::{AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::AutomataRegistry,
};

pub(in crate::plugin) fn remove_slices(
    mut commands: Commands,
    mut removed: EventReader<AutomatonRemoved>,
    owners: Query<(Entity, &AutomatonId, Option<&GpuGridSlice>)>,
    mut allocator: ResMut<AtlasAllocator>,
    registry: Res<AutomataRegistry>,
) {
    for event in removed.read() {
        for (entity, id, slice) in &owners {
            if *id != event.id {
                continue;
            }
            if let Some(slice) = slice {
                allocator.release(slice.offset, slice.size);
            }
            commands.entity(entity).despawn();
        }
    }
    if registry.list().is_empty() {
        *allocator = AtlasAllocator::default();
    }
}

pub(in crate::plugin) fn allocate_slices(
    mut commands: Commands,
    mut allocator: ResMut<AtlasAllocator>,
    mut events: EventReader<AutomatonAdded>,
    mut registry: ResMut<AutomataRegistry>,
    slices: Query<&GpuGridSlice>,
) {
    for event in events.read() {
        if slices.get(event.entity).is_ok() {
            continue;
        }
        let Some(info) = registry.get_mut(event.id) else {
            continue;
        };
        let size = info.slice.size;
        if let Some((layer, offset)) = allocator.allocate(size) {
            let slice = GpuGridSlice {
                layer,
                offset,
                size,
                depth: info.slice.depth.max(1),
                rule: info.name.clone(),
                rule_bits: 0,
            };
            commands.entity(event.entity).insert(slice.clone());
            info.slice = slice;
            info.world_offset = offset.as_vec2().extend(0.0) * info.voxel_size;
        } else {
            warn!("voxel atlas is full: {}", info.name);
        }
    }
}

pub(in crate::plugin) fn reset_atlas(mut allocator: ResMut<AtlasAllocator>) {
    *allocator = AtlasAllocator::default();
}
