use super::*;
use engine_gpu::VoxelRenderStatus;
use ui::lab::chrome::drawers::Comparison;
pub(super) fn update(world: &mut World) {
    world.resource_scope(|world, mut v: Mut<Verification>| run(world, &mut v));
}
fn run(world: &mut World, v: &mut Verification) {
    v.frames += 1;
    assert!(
        v.frames < 6000,
        "world qualification timed out at phase {}",
        v.phase
    );
    if !world.resource::<VoxelRenderStatus>().has_recorded_draw() {
        return;
    }
    let mut owners: Vec<_> = world
        .query::<(Entity, &ChannelId, &LeniaSimulation)>()
        .iter(world)
        .map(|(e, id, s)| (e, *id, s.clone()))
        .collect();
    owners.sort_by_key(|(_, id, _)| *id);
    if owners.len() != 2 {
        return;
    }
    match v.phase {
        0 => {
            if world.resource::<Workbench>().sessions.len() != 2 {
                return;
            }
            scenario::configure(world, &owners);
            let ids = world
                .resource::<Workbench>()
                .sessions
                .iter()
                .map(|s| s.channel)
                .collect::<Vec<_>>();
            scenario::select(world, [ids[0], ids[1]]);
            v.phase = 1;
        }
        1 => {
            let c = world.resource::<Comparison>();
            let Some(pair) = &c.capture.result else {
                return;
            };
            assert_eq!(pair[0].stamp.tick, 1);
            assert!(pair[0].field.compare(&pair[1].field).unwrap().maximum > 0.0);
            checkpoint::csv(world, v, "worlds-csv");
            scenario::capture(world, v, "worlds.png");
            v.phase = 2;
        }
        2 if v.captured.load(std::sync::atomic::Ordering::Acquire) => {
            scenario::select(world, [owners[0].1, owners[1].1]);
            v.phase = 3;
        }
        3 if checkpoint::compare(world, v, false) => {
            checkpoint::csv(world, v, "continuous-csv");
            checkpoint::save(world, v);
            v.old_owner = Some(owners[0].0);
            v.phase = 4;
        }
        4 if checkpoint::load(world, v) => {
            v.phase = 5;
        }
        5 if Some(owners[0].0) != v.old_owner
            && owners.iter().all(|(_, _, s)| s.target_tick == 4) =>
        {
            scenario::select(world, [owners[0].1, owners[1].1]);
            v.phase = 6;
        }
        6 if checkpoint::compare(world, v, true) => {
            scenario::capture(world, v, "continuous-comparison.png");
            v.phase = 7;
        }
        7 if v.captured.load(std::sync::atomic::Ordering::Acquire) => {
            std::fs::write(v.output.join("SUCCESS"),"Edited periodic worlds, undo/redo, completed CPU/GPU comparison, CSV, exact float project restoration, CPU oracle continuation and native screenshots passed.\n").unwrap();
            world.send_event(AppExit::Success);
            v.phase = 8;
        }
        _ => {}
    }
}
