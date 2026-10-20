//! Channel contract: ids, publishing order, selection repair and presets.
use super::*;

fn row(id: u32, backend: Backend) -> ChannelRow {
    ChannelRow {
        id: ChannelId(id),
        kind: "life:conway".into(),
        backend,
        status: Status::Paused,
        tick: 0,
        epoch: 0,
        settings: ChannelSettings::new(10.0),
    }
}

#[test]
fn ids_are_never_reused_and_capacity_is_shared() {
    let mut ids = ChannelIds::default();
    let first: Vec<_> = (0..ChannelIds::CAPACITY)
        .map(|_| ids.allocate().unwrap())
        .collect();
    assert_eq!(ids.allocate(), None, "ninth channel refused");
    ids.release(first[3]);
    assert_eq!(
        ids.allocate(),
        Some(ChannelId(8)),
        "released ids are not recycled"
    );
    assert_eq!(ids.open(), ChannelIds::CAPACITY);
    ids.release(ChannelId(999));
    assert_eq!(
        ids.open(),
        ChannelIds::CAPACITY,
        "unknown release is ignored"
    );
    assert_eq!(ChannelId::default(), ChannelId::UNASSIGNED);
}

#[test]
fn publish_replaces_one_backend_and_keeps_creation_order() {
    let mut channels = Channels::default();
    channels.publish(Backend::Gpu, [row(1, Backend::Gpu)]);
    channels.publish(Backend::Cpu, [row(2, Backend::Cpu), row(0, Backend::Cpu)]);
    let order: Vec<_> = channels.rows.iter().map(|r| r.id.0).collect();
    assert_eq!(order, [0, 1, 2]);
    channels.publish(Backend::Cpu, []);
    assert_eq!(channels.rows.len(), 1);
    assert!(!channels.is_full());
}

fn settle_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ChannelPlugin));
    app
}

fn publish_cpu(app: &mut App, ids: &[u32]) {
    let rows: Vec<_> = ids.iter().map(|id| row(*id, Backend::Cpu)).collect();
    app.world_mut()
        .resource_mut::<Channels>()
        .publish(Backend::Cpu, rows);
    app.update();
}

#[test]
fn settle_selects_predecessor_after_removal() {
    let mut app = settle_app();
    publish_cpu(&mut app, &[0, 2, 5]);
    app.world_mut().resource_mut::<Channels>().selected = Some(ChannelId(2));
    publish_cpu(&mut app, &[0, 5]);
    assert_eq!(
        app.world().resource::<Channels>().selected,
        Some(ChannelId(0))
    );
    publish_cpu(&mut app, &[5]);
    assert_eq!(
        app.world().resource::<Channels>().selected,
        Some(ChannelId(5)),
        "falls back to first"
    );
    publish_cpu(&mut app, &[]);
    assert_eq!(app.world().resource::<Channels>().selected, None);
}

#[test]
fn select_command_targets_only_existing_rows() {
    let mut app = settle_app();
    publish_cpu(&mut app, &[0, 1]);
    for (target, expected) in [(1, 1), (9, 1)] {
        app.world_mut().send_event(ChannelCommand {
            id: ChannelId(target),
            action: ChannelAction::Select,
        });
        app.update();
        assert_eq!(
            app.world().resource::<Channels>().selected,
            Some(ChannelId(expected))
        );
    }
}

#[test]
fn rate_presets_span_one_to_sixty_hertz() {
    assert_eq!(RATE_PRESETS, [1.0, 10.0, 30.0, 60.0]);
    assert_eq!(ChannelSettings::new(10.0).colour, SIGNAL);
}
