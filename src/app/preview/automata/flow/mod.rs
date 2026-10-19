use super::{capture, verification::Verification};
use bevy::prelude::*;
use engine_core::events::AutomataCommand;
use models::workbench::{catalog::Kind, plugin::Workbench};
use std::io::Write;

pub(super) fn update(world: &mut World) {
    world.resource_scope(|world, mut check: Mut<Verification>| {
        if check.index >= Kind::ALL.len() {
            return;
        }
        check.frames += 1;
        assert!(check.frames < 6000, "native family verification timed out");
        let kind = Kind::ALL[check.index];
        if check.phase == 0 {
            let mut bench = world.resource_mut::<Workbench>();
            let Some(session) = bench.sessions.first_mut() else {
                return;
            };
            assert_eq!(session.kind, kind);
            session.reset().expect("default input");
            session.advance();
            assert_eq!(session.ticks, 0, "spawn is paused");
            session.single_step();
            assert_eq!(session.ticks, 1, "single step advances once");
            session.reset().unwrap();
            let ticks = match kind {
                Kind::Rule30 | Kind::Rule110 => 32,
                Kind::Life => 4,
                Kind::Hpp => 12,
                Kind::Koch => 3,
                Kind::Boids => 64,
                Kind::Ants => 500,
                _ => 256,
            };
            for _ in 0..ticks {
                session.single_step();
            }
            assert!(session.ticks > 0);
            check.ticks = session.ticks;
            check.phase = 1;
            check.hold = check.frames;
        } else if check.phase == 1 && check.frames > check.hold + 30 {
            let bench = world.resource::<Workbench>();
            assert_eq!(
                bench.sessions[0].ticks, check.ticks,
                "paused state is stable"
            );
            capture::request(world, &mut check, kind.id());
            check.phase = 2;
        } else if check.phase == 2 && capture::finished(&check) {
            writeln!(
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(check.output.join("sessions.csv"))
                    .unwrap(),
                "{},{},verified",
                kind.id(),
                check.ticks
            )
            .unwrap();
            info!(
                "AUTOMATA_FAMILY_VERIFIED: {} tick {}",
                kind.id(),
                check.ticks
            );
            world.send_event(AutomataCommand::Clear);
            check.phase = 3;
        } else if check.phase == 3 {
            if !world.resource::<Workbench>().sessions.is_empty() {
                return;
            }
            check.index += 1;
            if check.index == Kind::ALL.len() {
                std::fs::write(
                    check.output.join("SUCCESS"),
                    "All 12 families spawned, stepped, paused, captured and cleared.\n",
                )
                .unwrap();
                info!("AUTOMATA_RUNTIME_VERIFIED: all 12 families passed");
                world.send_event(AppExit::Success);
            } else {
                world.send_event(AutomataCommand::SeedPattern {
                    id: Kind::ALL[check.index].id().into(),
                });
                check.phase = 0;
            }
        }
    });
}
