use super::*;
use engine_core::automata::{GpuGridSlice, lenia::LeniaSimulation};
use engine_gpu::{LeniaProbe, VoxelRenderStatus};
mod actions;
mod oracle;
mod readiness;

pub(super) fn update(world: &mut World) {
    world.resource_scope(|world, mut v: Mut<Verification>| run(world, &mut v));
}

fn run(world: &mut World, v: &mut Verification) {
    v.frame += 1;
    assert!(
        v.frame < 1800,
        "Lenia runtime verification timed out at phase {}",
        v.phase
    );
    if v.phase >= 10 || !world.resource::<VoxelRenderStatus>().has_recorded_draw() {
        return;
    }
    let probe = world.resource::<LeniaProbe>().clone();
    let owners: Vec<_> = world
        .query::<(Entity, &GpuGridSlice, &LeniaSimulation)>()
        .iter(world)
        .map(|(entity, slice, sim)| (entity, slice.clone(), sim.clone()))
        .collect();
    if v.validated {
        if capture::finished(v) {
            actions::advance(
                world,
                v,
                owners.first().map_or(Entity::PLACEHOLDER, |o| o.0),
            );
        }
        return;
    }
    if v.phase == 8 {
        if owners.is_empty() && probe.active_instances() == 0 {
            oracle::cleared(v);
            v.validated = true;
            capture::request(world, v);
        }
        return;
    }
    let Some((entity, slice, sim)) = owners.first() else {
        return;
    };
    assert_eq!(
        owners.len(),
        1,
        "qualification requires one isolated Lenia instance"
    );
    if !readiness::ready(world, v, *entity, slice, sim) {
        return;
    }
    if !v.pending {
        probe.request(*entity);
        v.pending = true;
        return;
    }
    let Some(snapshot) = probe.take(*entity) else {
        return;
    };
    let snapshot = snapshot.expect("Lenia runtime GPU readback");
    if snapshot.epoch < v.epoch || (snapshot.epoch == v.epoch && snapshot.tick < v.tick) {
        v.pending = false;
        return;
    }
    oracle::verify(v, &snapshot);
    v.validated = true;
    if [0, 3, 6, 7, 9].contains(&v.phase) {
        capture::request(world, v);
    }
}
