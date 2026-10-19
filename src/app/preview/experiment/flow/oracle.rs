use super::*;
pub(super) fn check(world: &mut World, v: &mut Verification, owner: Entity, probe: &LeniaProbe) {
    let Some(snapshot) = probe.take(owner) else {
        probe.request(owner);
        return;
    };
    let snapshot = snapshot.unwrap();
    let target = if v.phase == 4 { 4 } else { 6 };
    if snapshot.tick != target {
        probe.request(owner);
        return;
    }
    assert_eq!(
        snapshot.densities, v.expected,
        "restored GPU checkpoint and continuation"
    );
    if v.phase == 4 {
        for _ in 0..2 {
            v.expected =
                simulation_kernel::core::lenia::step(&v.expected, snapshot.size, snapshot.params)
                    .unwrap();
        }
        world.get_mut::<LeniaSimulation>(owner).unwrap().target_tick = 6;
        probe.request(owner);
        v.phase = 5;
    } else {
        v.phase = 6;
        v.frames = 0;
    }
}
