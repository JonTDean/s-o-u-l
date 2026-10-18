use super::*;
use engine_core::automata::lenia::{LeniaAction, LeniaCommand};
use simulation_kernel::core::lenia::{LeniaSeed, seed, step};

pub(super) fn ready(
    world: &mut World,
    v: &mut Verification,
    entity: Entity,
    slice: &GpuGridSlice,
    sim: &LeniaSimulation,
) -> bool {
    assert_eq!([slice.size.x, slice.size.y, slice.depth], [64, 64, 8]);
    if v.phase == 0 && v.first_owner.is_none() {
        v.first_owner = Some(entity);
        v.expected = seed([64, 64, 8], LeniaSeed::Blob);
    }
    if v.phase == 2 && v.frame - v.hold_since < 30 {
        return false;
    }
    if v.phase == 6 {
        if sim.target_tick < 8 {
            return false;
        }
        if !sim.paused {
            world.send_event(LeniaCommand {
                entity,
                action: LeniaAction::Pause(true),
            });
            return false;
        }
        if v.tick == 0 {
            v.tick = sim.target_tick;
            for _ in 0..v.tick {
                v.expected = step(&v.expected, [64, 64, 8], v.params).unwrap();
            }
        }
    }
    if v.phase == 9 {
        assert_ne!(
            Some(entity),
            v.first_owner,
            "stale owner reused after clear"
        );
    }
    sim.paused && sim.target_tick == v.tick && sim.epoch == v.epoch && sim.params == v.params
}
