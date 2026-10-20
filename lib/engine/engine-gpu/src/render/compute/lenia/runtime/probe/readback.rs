use super::super::state::Slot;
use super::{LeniaProbe, LeniaSnapshot};
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};
use simulation_kernel::core::lenia::LeniaParams;
use std::sync::Mutex;

struct Pending {
    entity: Entity,
    buffer: Buffer,
    snapshot: LeniaSnapshot,
}

#[derive(Resource, Default)]
pub(crate) struct PendingReadbacks(Mutex<Vec<Pending>>);

pub(crate) fn record(
    world: &World,
    encoder: &mut wgpu::CommandEncoder,
    entity: Entity,
    slot: &Slot,
    params: LeniaParams,
) {
    if !world
        .resource::<LeniaProbe>()
        .0
        .lock()
        .expect("Lenia probe lock")
        .requested
        .remove(&entity)
    {
        return;
    }
    let source = &slot.buffers.states[slot.buffers.current];
    let buffer = world
        .resource::<RenderDevice>()
        .create_buffer(&BufferDescriptor {
            label: Some("lenia.runtime.probe"),
            size: source.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    encoder.copy_buffer_to_buffer(source, 0, &buffer, 0, source.size());
    let snapshot = LeniaSnapshot {
        epoch: slot.epoch,
        tick: slot.buffers.tick,
        size: slot.buffers.size,
        params,
        densities: Vec::new(),
        continuous: slot.buffers.kernel.map(|kernel| {
            simulation_kernel::core::lenia::continuous::State {
                kernel,
                values: Vec::new(),
            }
        }),
    };
    world
        .resource::<PendingReadbacks>()
        .0
        .lock()
        .expect("Lenia readback lock")
        .push(Pending {
            entity,
            buffer,
            snapshot,
        });
}

pub(crate) fn complete(
    pending: Res<PendingReadbacks>,
    probe: Res<LeniaProbe>,
    device: Res<RenderDevice>,
) {
    for Pending {
        entity,
        buffer,
        mut snapshot,
    } in pending.0.lock().expect("Lenia readback lock").drain(..)
    {
        let mapped = buffer.clone();
        let destination = probe.0.clone();
        buffer.slice(..).map_async(MapMode::Read, move |result| {
            let result = result.map_err(|error| error.to_string()).and_then(|_| {
                let data = mapped.slice(..).get_mapped_range();
                let words: Vec<u32> = data
                    .chunks_exact(4)
                    .map(|word| u32::from_le_bytes(word.try_into().expect("density word")))
                    .collect();
                drop(data);
                mapped.unmap();
                if let Some(state) = &mut snapshot.continuous {
                    state.values = words.into_iter().map(f32::from_bits).collect();
                    state
                        .validate(snapshot.size.iter().map(|v| *v as usize).product())
                        .map_err(str::to_string)?;
                    snapshot.densities = state.bytes();
                } else {
                    snapshot.densities = words
                        .into_iter()
                        .map(u8::try_from)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| "GPU density outside 0..255".to_string())?;
                }
                Ok(snapshot)
            });
            destination
                .lock()
                .expect("Lenia probe lock")
                .results
                .insert(entity, result);
        });
    }
    device.wgpu_device().poll(wgpu::Maintain::Poll);
}
