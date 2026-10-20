use super::*;
use bevy::prelude::*;
use engine_core::automata::{channel::ChannelCommand, lenia::LeniaSimulation};
use models::workbench::plugin::Workbench;
pub fn poll(
    mut state: ResMut<super::super::Comparison>,
    bench: Res<Workbench>,
    sims: Query<(Entity, &ChannelId, &LeniaSimulation)>,
    mut commands: EventWriter<ChannelCommand>,
    #[cfg(feature = "gpu-compute")] probe: Res<engine_gpu::LeniaProbe>,
) {
    let lookup = |id| -> Result<Stamp, String> {
        if let Some(s) = bench.sessions.iter().find(|s| s.channel == id) {
            if !s.paused {
                return Err("Pause both channels before capture or stepping".into());
            }
            field::cpu(s)?;
            return Ok(Stamp::from_cpu(s));
        }
        let (_, _, s) = sims
            .iter()
            .find(|(_, i, _)| **i == id)
            .ok_or("Compared channel closed")?;
        if !s.paused {
            return Err("Pause both channels before capture or stepping".into());
        }
        Ok(Stamp::from_gpu(id, s))
    };
    if let Some(step) = state.capture.request.take() {
        state.capture.result = None;
        let result = super::begin(&mut state, step, lookup, &mut commands);
        if let Err(e) = result {
            state.status = e;
            state.capture.pending = None;
        }
    }
    let Some(pending) = &state.capture.pending else {
        return;
    };
    let result = (|| -> Result<Option<[Frame; 2]>, String> {
        if pending.started.elapsed().as_secs() > 15 {
            return Err("Comparison timed out waiting for completed GPU work".into());
        }
        let mut frames = Vec::new();
        for expected in &pending.expected {
            let current = lookup(expected.id)?;
            if current.epoch != expected.epoch
                || current.tick > expected.tick
                || current.params != expected.params
                || current.kernel != expected.kernel
            {
                return Err("Comparison cancelled: channel changed during capture".into());
            }
            if current.tick < expected.tick {
                return Ok(None);
            }
            let field = if let Some(s) = bench.sessions.iter().find(|s| s.channel == expected.id) {
                field::cpu(s)?
            } else {
                #[cfg(feature = "gpu-compute")]
                {
                    let (entity, _, _) = sims
                        .iter()
                        .find(|(_, id, _)| **id == expected.id)
                        .ok_or("Compared owner closed")?;
                    match probe.latest(entity) {
                        Some(Ok(s))
                            if s.epoch == expected.epoch
                                && s.tick == expected.tick
                                && Some(s.params) == expected.params
                                && s.continuous.as_ref().map(|c| c.kernel) == expected.kernel =>
                        {
                            field::gpu(s)
                        }
                        Some(Err(e)) => return Err(e),
                        _ => {probe.request(entity); return Ok(None)}
                    }
                }
                #[cfg(not(feature="gpu-compute"))]
                {return Err("GPU comparison requires gpu-compute".into())}
            };
            frames.push(Frame {stamp:expected.clone(),field});
        }
        Ok(Some(frames.try_into().expect("two comparison frames")))
    })();
    match result {
        Ok(Some(pair)) => {state.capture.result=Some(pair); state.capture.pending=None; state.status="Captured both completed states".into();}
        Err(e) => {state.capture.pending=None;state.status=e;}
        Ok(None) => {}
    }
}
