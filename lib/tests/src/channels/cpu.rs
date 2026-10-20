//! Every ChannelAction against real CPU sessions, with simulated wall time.
use bevy::{prelude::*, state::app::StatesPlugin, time::TimeUpdateStrategy};
use engine_core::{
    automata::channel::{
        ChannelAction, ChannelCommand, ChannelId, ChannelIds, ChannelRow, Channels, SWATCHES,
        Status,
    },
    events::AutomataCommand,
    prelude::AppState,
};
use models::workbench::{
    experiment::config::Settings,
    operations::channels::ConfigureSession,
    plugin::{Workbench, WorkbenchPlugin},
};
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_event::<AutomataCommand>()
        .add_plugins(WorkbenchPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::InGame);
    app.update();
    app.update();
    app
}

fn spawn(app: &mut App, id: &str) -> Option<ChannelId> {
    let before = app.world().resource::<Channels>().rows.len();
    app.world_mut()
        .send_event(AutomataCommand::SeedPattern { id: id.into() });
    app.update();
    let channels = app.world().resource::<Channels>();
    (channels.rows.len() > before).then(|| channels.rows.last().unwrap().id)
}

fn send(app: &mut App, id: ChannelId, action: ChannelAction) {
    app.world_mut().send_event(ChannelCommand { id, action });
    app.update();
}

fn row(app: &App, id: ChannelId) -> ChannelRow {
    let channels = app.world().resource::<Channels>();
    channels.get(id).cloned().expect("row published")
}

#[test]
fn spawn_publishes_a_paused_selected_row() {
    let mut app = app();
    let id = spawn(&mut app, "life:conway").unwrap();
    let r = row(&app, id);
    assert_eq!(
        (r.status, r.tick, r.kind.as_str()),
        (Status::Paused, 0, "life:conway")
    );
    assert_eq!(r.settings.rate_hz, 10.0);
    assert_eq!(app.world().resource::<Channels>().selected, Some(id));
}

#[test]
fn running_channels_advance_at_their_own_rate() {
    let mut app = app();
    let fast = spawn(&mut app, "life:conway").unwrap();
    let slow = spawn(&mut app, "wolfram:rule30").unwrap();
    send(&mut app, fast, ChannelAction::SetRate(60.0));
    send(&mut app, slow, ChannelAction::SetRate(1.0));
    send(&mut app, fast, ChannelAction::Pause(false));
    send(&mut app, slow, ChannelAction::Pause(false));
    let (f0, s0) = (row(&app, fast).tick, row(&app, slow).tick);
    for _ in 0..10 {
        app.update();
    }
    assert!((59..=61).contains(&(row(&app, fast).tick - f0)));
    assert!((0..=1).contains(&(row(&app, slow).tick - s0)));
    assert_eq!(row(&app, fast).status, Status::Running);
    send(&mut app, fast, ChannelAction::SetRate(500.0));
    assert_eq!(
        row(&app, fast).settings.rate_hz,
        60.0,
        "rates clamp to 1–60"
    );
}

#[test]
fn step_only_while_paused_and_reset_bumps_epoch() {
    let mut app = app();
    let id = spawn(&mut app, "life:conway").unwrap();
    send(&mut app, id, ChannelAction::Step);
    assert_eq!(row(&app, id).tick, 1);
    send(&mut app, id, ChannelAction::Reset);
    let r = row(&app, id);
    assert_eq!((r.tick, r.status, r.epoch), (0, Status::Paused, 1));
    send(&mut app, id, ChannelAction::Pause(false));
    let before = row(&app, id).tick;
    send(&mut app, id, ChannelAction::Step);
    assert_eq!(
        row(&app, id).tick,
        before + 1,
        "only the 10 Hz clock advanced"
    );
}

#[test]
fn remove_releases_shared_capacity() {
    let mut app = app();
    let ids: Vec<_> = (0..8)
        .map(|_| spawn(&mut app, "wolfram:rule30").unwrap())
        .collect();
    assert_eq!(spawn(&mut app, "wolfram:rule30"), None);
    assert!(app.world().resource::<Workbench>().error.is_some());
    send(&mut app, ids[2], ChannelAction::Remove);
    assert!(app.world().resource::<Channels>().get(ids[2]).is_none());
    let again = spawn(&mut app, "wolfram:rule30").unwrap();
    assert!(again.0 > ids[7].0, "ids are not reused");
}

#[test]
fn colour_input_and_configuration_apply() {
    let mut app = app();
    let id = spawn(&mut app, "regular:nfa").unwrap();
    send(&mut app, id, ChannelAction::SetColour(SWATCHES[3]));
    send(&mut app, id, ChannelAction::SetInput("ab".into()));
    send(&mut app, id, ChannelAction::Reset);
    assert_eq!(row(&app, id).settings.colour, SWATCHES[3]);
    assert_eq!(app.world().resource::<Workbench>().sessions[0].input, "ab");
    let life = spawn(&mut app, "life:conway").unwrap();
    let mut settings = Settings::for_kind(models::workbench::catalog::Kind::Life);
    settings.random_seed = 7;
    app.world_mut().send_event(ConfigureSession {
        id: life,
        settings: Box::new(settings),
        reset: false,
    });
    app.update();
    let note = &app.world().resource::<Workbench>().sessions[1].note;
    assert!(
        note.as_deref().is_some_and(|n| n.contains("reset")),
        "{note:?}"
    );
    send(&mut app, ChannelId(999), ChannelAction::Remove);
    assert_eq!(app.world().resource::<Channels>().rows.len(), 2);
}

#[test]
fn halted_machines_report_halted_and_refuse_resume() {
    let mut app = app();
    let id = spawn(&mut app, "regular:nfa").unwrap();
    for _ in 0..8 {
        send(&mut app, id, ChannelAction::Step);
    }
    assert_eq!(row(&app, id).status, Status::Halted);
    send(&mut app, id, ChannelAction::Pause(false));
    assert_eq!(row(&app, id).status, Status::Halted);
}

#[test]
fn replaced_sessions_are_adopted_and_old_ids_released() {
    let mut app = app();
    spawn(&mut app, "life:conway");
    spawn(&mut app, "swarm:boids");
    let fresh = models::workbench::session::Session::new(models::workbench::catalog::Kind::Ants);
    app.world_mut().resource_mut::<Workbench>().sessions = vec![fresh];
    app.update();
    let channels = app.world().resource::<Channels>();
    assert_eq!(channels.rows.len(), 1);
    assert_eq!(channels.rows[0].kind, "swarm:ants");
    assert_eq!(app.world().resource::<ChannelIds>().open(), 1);
    app.world_mut().send_event(AutomataCommand::Clear);
    app.update();
    app.update();
    assert_eq!(app.world().resource::<ChannelIds>().open(), 0);
}
