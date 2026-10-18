use super::*;
use engine_core::{
    automata::lenia::{LeniaAction, LeniaCommand},
    events::AutomataCommand,
};
use simulation_kernel::core::lenia::{LeniaSeed, seed, step};

pub(super) fn advance(world: &mut World, v: &mut Verification, entity: Entity) {
    let send = |world: &mut World, action| {
        world.send_event(LeniaCommand { entity, action });
    };
    match v.phase {
        0 => {
            send(world, LeniaAction::Step);
            v.expected = step(&v.expected, [64, 64, 8], v.params).unwrap();
            v.tick = 1;
        }
        1 => v.hold_since = v.frame,
        2 => {
            v.params = LeniaParams {
                mu: 0.27,
                sigma: 0.08,
                dt: 0.02,
            };
            send(world, LeniaAction::Parameters(v.params));
            send(world, LeniaAction::Step);
            v.expected = step(&v.expected, [64, 64, 8], v.params).unwrap();
            v.tick = 2;
        }
        3 => {
            send(world, LeniaAction::Reset);
            v.expected = seed([64, 64, 8], LeniaSeed::Blob);
            v.epoch += 1;
            v.tick = 0;
        }
        4 => {
            send(world, LeniaAction::Seed(LeniaSeed::Ring));
            v.expected = seed([64, 64, 8], LeniaSeed::Ring);
            v.epoch += 1;
        }
        5 => send(world, LeniaAction::Pause(false)),
        6 => {
            send(world, LeniaAction::Reset);
            v.expected = seed([64, 64, 8], LeniaSeed::Ring);
            v.epoch += 1;
            v.tick = 0;
        }
        7 => {
            world.send_event(AutomataCommand::Clear);
        }
        8 => {
            world.send_event(AutomataCommand::SeedPattern {
                id: "lenia:orbium".into(),
            });
            v.epoch = 0;
            v.params = LeniaParams::default();
        }
        9 => {
            std::fs::write(
                v.output.join("SUCCESS"),
                "CPU/GPU parity, controls, reset, clear, and respawn verified\n",
            )
            .unwrap();
            info!("LENIA_RUNTIME_VERIFIED: all workbench phases passed");
            world.send_event(AppExit::Success);
        }
        _ => unreachable!(),
    }
    v.phase += 1;
    v.pending = false;
    v.validated = false;
}
