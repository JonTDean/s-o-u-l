use super::super::super::runtime::{LiveSimulations, state::synchronize};
use super::super::device::Gpu;
use bevy::prelude::*;
use engine_core::automata::{GpuGridSlice, lenia::LeniaSimulation};
use simulation_kernel::core::lenia::{LeniaSeed, seed};
use std::collections::HashMap;

#[test]
fn gpu_lenia_reset_removal_and_entity_reuse_discard_prior_state() {
    let gpu = Gpu::new();
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let slice = GpuGridSlice {
        layer: 2,
        offset: UVec2::new(4, 8),
        size: UVec2::new(7, 5),
        depth: 2,
        rule: "lenia".into(),
        rule_bits: 0,
    };
    let mut live = LiveSimulations(vec![(entity, slice, LeniaSimulation::default())]);
    let mut slots = HashMap::new();
    synchronize(&mut slots, &live, &gpu.device, &gpu.pipelines);
    let initial = seed([7, 5, 2], LeniaSeed::Blob);
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    slots
        .get_mut(&entity)
        .unwrap()
        .buffers
        .advance(
            &mut encoder,
            &gpu.queue,
            &gpu.pipelines,
            3,
            Default::default(),
        )
        .unwrap();
    gpu.queue.submit([encoder.finish()]);
    assert_ne!(gpu.values(&slots[&entity].buffers), initial);
    live.0[0].2.reset();
    synchronize(&mut slots, &live, &gpu.device, &gpu.pipelines);
    assert_eq!((slots[&entity].epoch, slots[&entity].buffers.tick), (1, 0));
    assert_eq!(gpu.values(&slots[&entity].buffers), initial);
    let (_, slice, _) = live.0.pop().unwrap();
    synchronize(&mut slots, &live, &gpu.device, &gpu.pipelines);
    assert!(slots.is_empty());
    world.despawn(entity);
    let replacement = world.spawn_empty().id();
    assert_ne!(replacement, entity);
    live.0.push((
        replacement,
        slice,
        LeniaSimulation {
            seed: LeniaSeed::Ring,
            ..default()
        },
    ));
    synchronize(&mut slots, &live, &gpu.device, &gpu.pipelines);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[&replacement].buffers.tick, 0);
    assert_eq!(
        gpu.values(&slots[&replacement].buffers),
        seed([7, 5, 2], LeniaSeed::Ring)
    );
    live.0[0].1.size = UVec2::splat(u32::MAX);
    live.0[0].1.depth = u32::MAX;
    synchronize(&mut slots, &live, &gpu.device, &gpu.pipelines);
    assert!(slots.is_empty());
}
