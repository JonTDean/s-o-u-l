use super::super::LeniaPipelines;
use super::{GpuSimulations, LiveSimulations, state::synchronize};
use crate::plugin::GlobalVoxelAtlas;
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::Origin3d,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

pub(super) fn run(context: &mut RenderContext, world: &World) {
    let images = world.resource::<RenderAssets<GpuImage>>();
    let atlas = world.resource::<GlobalVoxelAtlas>();
    let Some(image) = images.get(&atlas.atlas) else {
        return;
    };
    let device = world.resource::<RenderDevice>();
    let queue = world.resource::<RenderQueue>();
    let pipelines = world.resource::<LeniaPipelines>();
    let live = world.resource::<LiveSimulations>();
    let mut slots = world
        .resource::<GpuSimulations>()
        .0
        .lock()
        .expect("Lenia state lock");
    synchronize(&mut slots, live, device, pipelines);
    for (entity, slice, sim) in &live.0 {
        let Some(slot) = slots.get_mut(entity) else {
            continue;
        };
        let steps = sim.target_tick.saturating_sub(slot.buffers.tick).min(5) as u32;
        if let Err(error) = slot.buffers.advance(
            context.command_encoder(),
            queue,
            pipelines,
            steps,
            sim.params,
        ) {
            warn!("Lenia parameters rejected: {error}");
            continue;
        }
        slot.buffers.present(
            context.command_encoder(),
            pipelines,
            &image.texture,
            Origin3d {
                x: slice.offset.x,
                y: slice.offset.y,
                z: slice.layer,
            },
        );
        super::probe::record(world, context.command_encoder(), *entity, slot, sim.params);
    }
}
