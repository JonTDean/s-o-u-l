use super::{
    super::{LeniaBuffers, LeniaPipelines},
    LiveSimulations,
};
use bevy::{prelude::*, render::renderer::RenderDevice};
use std::{collections::HashMap, sync::Mutex};

pub(crate) struct Slot {
    pub epoch: u64,
    pub origin: [u32; 3],
    pub buffers: LeniaBuffers,
}

#[derive(Resource, Default)]
pub(crate) struct GpuSimulations(pub Mutex<HashMap<Entity, Slot>>);

pub(super) fn synchronize(
    slots: &mut HashMap<Entity, Slot>,
    live: &LiveSimulations,
    device: &RenderDevice,
    pipelines: &LeniaPipelines,
) {
    slots.retain(|entity, _| live.0.iter().any(|(owner, _, _)| owner == entity));
    for (entity, slice, sim) in &live.0 {
        let size = [slice.size.x, slice.size.y, slice.depth.max(1)];
        let origin = [slice.offset.x, slice.offset.y, slice.layer];
        if slots.get(entity).is_some_and(|slot| {
            slot.epoch == sim.epoch && slot.origin == origin && slot.buffers.size == size
        }) {
            continue;
        }
        slots.remove(entity);
        let cells = simulation_kernel::core::lenia::seed(size, sim.seed);
        match LeniaBuffers::new(device, pipelines, size, &cells) {
            Ok(buffers) => {
                slots.insert(
                    *entity,
                    Slot {
                        epoch: sim.epoch,
                        origin,
                        buffers,
                    },
                );
            }
            Err(error) => warn!("Lenia allocation rejected: {error}"),
        }
    }
}
