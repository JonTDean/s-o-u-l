use super::*;
pub(super) fn update(world: &mut World) {
    world.resource_scope(|world, mut v: Mut<Verification>| {
        v.frames += 1;
        assert!(v.frames < 12000, "authoring timeout at phase {}", v.phase);
        if v.frames < v.wait {
            return;
        }
        if v.phase <= 3 || v.phase == 12 {
            cpu::run(world, &mut v);
        } else {
            gpu::run(world, &mut v);
        }
    });
}
