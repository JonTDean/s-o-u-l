//! Readback must represent the exact requested epoch and tick before saving.
use super::*;
use engine_core::automata::lenia::SavedLenia;
use engine_gpu::LeniaProbe;
pub(super) fn poll(
    files: &mut Files,
    probe: &LeniaProbe,
    sims: &Query<(Entity, &LeniaSimulation)>,
) {
    let Some(mut pending) = files.save.take() else {
        return;
    };
    if pending.started.elapsed().as_secs() > 15 {
        files.status = "Save timed out waiting for GPU readback".into();
        return;
    }
    let mut waiting = Vec::new();
    for (entity, expected) in pending.owners {
        let Ok((_, current)) = sims.get(entity) else {
            files.status = "Save cancelled: instance removed".into();
            return;
        };
        if !current.paused
            || current.epoch != expected.epoch
            || current.target_tick != expected.target_tick
            || current.kernel != expected.kernel
            || current.params != expected.params
        {
            files.status = "Save cancelled: simulation changed during capture".into();
            return;
        }
        match probe.take(entity) {
            Some(Ok(snapshot))
                if snapshot.epoch == expected.epoch
                    && snapshot.tick == expected.target_tick
                    && snapshot.params == expected.params =>
            {
                pending.project.lenia.push(SavedLenia {
                    continuous: snapshot.continuous,
                    size: snapshot.size,
                    tick: snapshot.tick,
                    params: snapshot.params,
                    seed: expected.seed,
                    paused: expected.paused,
                    densities: snapshot.densities,
                });
            }
            Some(Err(error)) => {
                files.status = error;
                return;
            }
            _ => {
                probe.request(entity);
                waiting.push((entity, expected));
            }
        }
    }
    pending.owners = waiting;
    if pending.owners.is_empty() {
        files.status = match models::workbench::project::write(&pending.path, &pending.project) {
            Ok(()) => format!("Saved {}", pending.path.display()),
            Err(e) => e,
        };
    } else {
        files.save = Some(pending);
    }
}
