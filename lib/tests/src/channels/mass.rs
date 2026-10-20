//! Mass adapter behaviour against a scripted mailbox; no GPU required.
use bevy::{prelude::*, time::TimeUpdateStrategy};
use engine_core::automata::{
    channel::{
        ChannelAction, ChannelCommand, ChannelId, ChannelPlugin, ChannelSet, ChannelSettings,
        metrics::{MetricAvailability, Metrics},
    },
    lenia::{LeniaSimulation, ReadbackBusy},
};
use engine_render::command_executor::{
    channels::publish,
    metrics::{MASS, MassSampling, SnapshotMailbox, sample, toggle},
};
use std::{sync::Mutex, time::Duration};

type Snap = Result<(u64, u64, Vec<u8>), String>;

#[derive(Resource, Default)]
struct Fake {
    requests: Mutex<Vec<Entity>>,
    ready: Mutex<Option<Snap>>,
}
impl SnapshotMailbox for Fake {
    fn request(&self, owner: Entity) {
        self.requests.lock().unwrap().push(owner);
    }
    fn take(&self, _owner: Entity) -> Option<Snap> {
        self.ready.lock().unwrap().take()
    }
}

fn app() -> (App, ChannelId) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ChannelPlugin))
        .init_resource::<Fake>()
        .init_resource::<MassSampling>()
        .init_resource::<ReadbackBusy>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_systems(
            Update,
            (publish, toggle, sample::<Fake>)
                .chain()
                .in_set(ChannelSet::Publish),
        );
    let id = ChannelId(4);
    app.world_mut()
        .spawn((id, ChannelSettings::new(60.0), LeniaSimulation::default()));
    app.update();
    (app, id)
}

fn availability(app: &App, id: ChannelId) -> MetricAvailability {
    let metrics = app.world().resource::<Metrics>();
    metrics.get(id).unwrap().availability.clone()
}

fn enable(app: &mut App, id: ChannelId) {
    app.world_mut().send_event(ChannelCommand {
        id,
        action: ChannelAction::SampleMetric(true),
    });
}

fn ready(app: &App, snap: Snap) {
    *app.world().resource::<Fake>().ready.lock().unwrap() = Some(snap);
}

fn requests(app: &App) -> usize {
    app.world()
        .resource::<Fake>()
        .requests
        .lock()
        .unwrap()
        .len()
}

fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

#[test]
fn off_by_default_declares_opt_in_and_never_requests() {
    let (mut app, id) = app();
    run(&mut app, 10);
    assert_eq!(availability(&app, id), MetricAvailability::OptIn);
    assert_eq!(requests(&app), 0);
}

#[test]
fn enabled_requests_are_throttled_and_results_recorded() {
    let (mut app, id) = app();
    enable(&mut app, id);
    run(&mut app, 10);
    assert_eq!(
        requests(&app),
        1,
        "waiting for a result blocks further requests"
    );
    ready(&app, Ok((0, 3, vec![255; 4])));
    app.update();
    let metric = app.world().resource::<Metrics>().get(id).unwrap().clone();
    assert_eq!(metric.id, MASS);
    assert_eq!(metric.latest.unwrap().value, 4.0);
    assert!(matches!(
        metric.availability,
        MetricAvailability::Sampled { .. }
    ));
}

#[test]
fn readback_error_becomes_unavailable() {
    let (mut app, id) = app();
    enable(&mut app, id);
    run(&mut app, 4);
    ready(&app, Err("map failed".into()));
    app.update();
    assert_eq!(
        availability(&app, id),
        MetricAvailability::Unavailable("map failed".into())
    );
}

#[test]
fn stalled_readback_times_out() {
    let (mut app, id) = app();
    enable(&mut app, id);
    run(&mut app, 30);
    assert_eq!(
        availability(&app, id),
        MetricAvailability::Unavailable("GPU readback timed out".into())
    );
}

#[test]
fn stale_epoch_snapshots_are_dropped() {
    let (mut app, id) = app();
    let mut owners = app.world_mut().query::<&mut LeniaSimulation>();
    owners.single_mut(app.world_mut()).unwrap().epoch = 2;
    enable(&mut app, id);
    run(&mut app, 4);
    ready(&app, Ok((1, 9, vec![255; 4])));
    app.update();
    assert!(
        app.world()
            .resource::<Metrics>()
            .get(id)
            .unwrap()
            .latest
            .is_none()
    );
}

#[test]
fn busy_mailbox_suspends_sampling() {
    let (mut app, id) = app();
    app.world_mut().resource_mut::<ReadbackBusy>().0 = true;
    enable(&mut app, id);
    run(&mut app, 10);
    assert_eq!(requests(&app), 0, "project save owns the mailbox");
    ready(&app, Ok((0, 1, vec![255; 4])));
    app.update();
    assert!(
        app.world()
            .resource::<Fake>()
            .ready
            .lock()
            .unwrap()
            .is_some(),
        "not taken"
    );
}
