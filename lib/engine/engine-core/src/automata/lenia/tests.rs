use super::*;
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};

fn send(app: &mut App, entity: Entity, action: LeniaAction) {
    app.world_mut().send_event(LeniaCommand { entity, action });
    app.update();
}

#[test]
fn lenia_controls_pause_step_reset_parameters_and_ignore_stale_entities() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, LeniaControlPlugin));
    let owner = app.world_mut().spawn(LeniaSimulation::default()).id();
    send(&mut app, owner, LeniaAction::Step);
    assert_eq!(
        app.world()
            .get::<LeniaSimulation>(owner)
            .unwrap()
            .target_tick,
        0
    );
    send(&mut app, owner, LeniaAction::Pause(true));
    send(&mut app, owner, LeniaAction::Step);
    send(&mut app, owner, LeniaAction::Step);
    assert_eq!(
        app.world()
            .get::<LeniaSimulation>(owner)
            .unwrap()
            .target_tick,
        2
    );
    let changed = LeniaParams {
        mu: 0.4,
        sigma: 0.08,
        dt: 0.02,
    };
    send(&mut app, owner, LeniaAction::Parameters(changed));
    send(
        &mut app,
        owner,
        LeniaAction::Parameters(LeniaParams {
            sigma: 0.0,
            ..changed
        }),
    );
    assert_eq!(
        app.world().get::<LeniaSimulation>(owner).unwrap().params,
        changed
    );
    send(&mut app, owner, LeniaAction::Reset);
    let sim = app.world().get::<LeniaSimulation>(owner).unwrap();
    assert_eq!((sim.target_tick, sim.epoch, sim.paused), (0, 1, true));
    assert_eq!(sim.params, changed);
    send(&mut app, owner, LeniaAction::Seed(LeniaSeed::Ring));
    let sim = app.world().get::<LeniaSimulation>(owner).unwrap();
    assert_eq!((sim.seed, sim.epoch), (LeniaSeed::Ring, 2));
    send(&mut app, owner, LeniaAction::Pause(false));
    assert!(!app.world().get::<LeniaSimulation>(owner).unwrap().paused);
    app.world_mut().despawn(owner);
    let replacement = app.world_mut().spawn(LeniaSimulation::default()).id();
    send(&mut app, owner, LeniaAction::Pause(true));
    assert!(
        !app.world()
            .get::<LeniaSimulation>(replacement)
            .unwrap()
            .paused
    );
}
