//! Cross-crate production routing and per-instance lifecycle verification.
use bevy::prelude::*;
use engine_core::events::AutomataCommand;
use models::{
    ComputationalIntelligencePlugin,
    workbench::{
        catalog::{Catalog, Kind},
        plugin::Workbench,
        session::{Model, Session},
    },
};

#[test]
fn every_family_plugin_registers_and_native_spawn_clear_commands_execute() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ComputationalIntelligencePlugin));
    let catalog = app.world().resource::<Catalog>();
    assert_eq!(catalog.0.len(), Kind::ALL.len());
    for kind in Kind::ALL {
        assert!(app.world().resource::<Catalog>().0.contains(&kind));
        app.world_mut().send_event(AutomataCommand::SeedPattern {
            id: kind.id().into(),
        });
        app.update();
        let bench = app.world().resource::<Workbench>();
        assert_eq!(bench.sessions.len(), 1);
        assert_eq!(bench.sessions[0].kind, kind);
        app.world_mut().send_event(AutomataCommand::Clear);
        app.update();
        assert!(app.world().resource::<Workbench>().sessions.is_empty());
    }
}

#[test]
fn all_families_pause_step_reset_and_hold_terminal_states() {
    for kind in Kind::ALL {
        let mut session = Session::new(kind);
        session.advance();
        assert_eq!(session.ticks, 0);
        session.single_step();
        assert_eq!(session.ticks, 1);
        session.paused = false;
        session.single_step();
        assert_eq!(session.ticks, 1);
        session.advance();
        assert_eq!(session.ticks, 2);
        session.reset().unwrap();
        assert_eq!(session.ticks, 0);
        assert!(session.paused);
        if let Model::Symbolic(_) = &session.model {
            for _ in 0..1000 {
                session.step();
            }
            assert!(session.halted());
            let halted_tick = session.ticks;
            session.step();
            assert_eq!(session.ticks, halted_tick);
        }
    }
}

#[test]
fn capacity_and_invalid_inputs_do_not_mutate_other_sessions() {
    let mut bench = Workbench::default();
    for kind in Kind::ALL {
        bench.spawn(kind);
    }
    assert_eq!(bench.sessions.len(), 8);
    assert!(bench.error.is_some());
    bench.sessions[0].single_step();
    assert!(bench.sessions[1..].iter().all(|s| s.ticks == 0));
    let mut machine = Session::new(Kind::Nfa);
    machine.single_step();
    machine.input = "a".repeat(257);
    assert!(machine.reset().is_err());
    assert_eq!(machine.ticks, 1);
    bench.clear();
    bench.spawn(Kind::Ants);
    assert_eq!(bench.sessions.len(), 1);
    assert_eq!(bench.sessions[0].ticks, 0);
}
