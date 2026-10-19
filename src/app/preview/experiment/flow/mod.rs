mod oracle;
use super::*;
use engine_core::automata::{GpuGridSlice, lenia::LeniaSimulation};
use engine_gpu::{LeniaProbe, VoxelRenderStatus};
use models::workbench::{
    experiment::{config::Pattern, metrics},
    plugin::Workbench,
};
pub(super) fn update(world: &mut World) {
    world.resource_scope(|world, mut v: Mut<Verification>| run(world, &mut v));
}
fn run(world: &mut World, v: &mut Verification) {
    v.frames += 1;
    assert!(
        v.frames < 6000,
        "Experiment verification timed out at {}",
        v.phase
    );
    if v.phase == 7 {
        if v.captured.load(std::sync::atomic::Ordering::Acquire) {
            std::fs::write(v.output.join("SUCCESS"),"Parameters, comparison, atomic project restore, GPU continuation and export verified.\n").unwrap();
            world.send_event(AppExit::Success);
            v.phase = 8;
        }
        return;
    }
    if v.phase >= 8 || !world.resource::<VoxelRenderStatus>().has_recorded_draw() {
        return;
    }
    let owners: Vec<_> = world
        .query::<(Entity, &GpuGridSlice, &LeniaSimulation)>()
        .iter(world)
        .map(|(e, _, s)| (e, s.clone()))
        .collect();
    let Some((owner, sim)) = owners.first() else {
        return;
    };
    let probe = world.resource::<LeniaProbe>().clone();
    match v.phase {
        0 => {
            let mut bench = world.resource_mut::<Workbench>();
            if bench.sessions.len() != 2 {
                return;
            }
            let mut settings = bench.sessions[0].settings.clone();
            settings.pattern = Pattern::Blinker;
            bench.sessions[0].configure(settings, true).unwrap();
            let mut settings = bench.sessions[1].settings.clone();
            settings.elementary = 90;
            bench.sessions[1].configure(settings, false).unwrap();
            for _ in 0..12 {
                metrics::step_pair(&mut bench.sessions, 0, 1).unwrap();
            }
            world
                .get_mut::<LeniaSimulation>(*owner)
                .unwrap()
                .target_tick = 4;
            v.owner = Some(*owner);
            probe.request(*owner);
            v.phase = 1;
        }
        1 => {
            let Some(snapshot) = probe.take(*owner) else {
                probe.request(*owner);
                return;
            };
            let snapshot = snapshot.unwrap();
            if snapshot.tick != 4 {
                probe.request(*owner);
                return;
            }
            v.expected = snapshot.densities;
            checkpoint::save(world, v, *owner, sim.clone());
            v.phase = 2;
        }
        2 => {
            if checkpoint::load(world, v) {
                v.phase = 3;
            }
        }
        3 => {
            if Some(*owner) != v.owner && sim.target_tick == 4 {
                v.owner = Some(*owner);
                probe.request(*owner);
                v.phase = 4;
            }
        }
        4 | 5 => oracle::check(world, v, *owner, &probe),
        6 if v.frames > 45 => {
            capture::request(world, v);
            v.phase = 7;
        }
        _ => {}
    }
}
