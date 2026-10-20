use super::*;
use engine_core::{
    automata::{
        channel::{Backend, ChannelAction, ChannelCommand, ChannelIds, Channels, SWATCHES, Status},
        lenia::{LeniaAction, LeniaCommand},
    },
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
        // Phase 10 is the channel-command pass; it writes SUCCESS when it finishes.
        9 => v.channel_pass = (0, v.frame),
        _ => unreachable!(),
    }
    v.phase += 1;
    v.pending = false;
    v.validated = false;
}

/// Step, rate, colour and removal of the respawned instance through the channel contract.
pub(super) fn channel_pass(world: &mut World, v: &mut Verification) {
    let (step, since) = v.channel_pass;
    if v.frame < since + 2 {
        return; // let routed commands land and rows republish
    }
    let gpu = |world: &World| {
        let rows = &world.resource::<Channels>().rows;
        rows.iter().find(|r| r.backend == Backend::Gpu).cloned()
    };
    let target = |world: &mut World| {
        let mut sims = world.query::<&LeniaSimulation>();
        sims.iter(world).next().map(|s| s.target_tick)
    };
    match step {
        0 => {
            let row = gpu(world).expect("respawned Lenia channel");
            assert_eq!(row.status, Status::Paused, "respawn is paused");
            v.channel_tick = target(world).expect("Lenia owner");
            for action in [
                ChannelAction::Step,
                ChannelAction::SetRate(10.0),
                ChannelAction::SetColour(SWATCHES[3]),
            ] {
                world.send_event(ChannelCommand { id: row.id, action });
            }
        }
        1 => {
            let row = gpu(world).expect("Lenia channel");
            assert_eq!(
                target(world),
                Some(v.channel_tick + 1),
                "channel Step advanced once"
            );
            assert_eq!(row.settings.rate_hz, 10.0, "row rate updated");
            assert_eq!(row.settings.colour, SWATCHES[3], "row colour updated");
            world.send_event(ChannelCommand {
                id: row.id,
                action: ChannelAction::Remove,
            });
        }
        _ => {
            assert!(gpu(world).is_none(), "removed channel left its row");
            assert_eq!(
                world.resource::<ChannelIds>().open(),
                0,
                "removal released the id"
            );
            std::fs::write(
                v.output.join("SUCCESS"),
                "CPU/GPU parity, controls, reset, clear, respawn and channel commands verified\n",
            )
            .unwrap();
            info!("LENIA_CHANNEL_VERIFIED: step, rate, colour and removal through ChannelCommand");
            info!("LENIA_RUNTIME_VERIFIED: all workbench phases passed");
            world.send_event(AppExit::Success);
            v.phase = 11;
        }
    }
    v.channel_pass = (step + 1, v.frame);
}
