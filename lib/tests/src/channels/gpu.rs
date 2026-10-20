//! Lenia channel routing without a GPU: owners are plain entities with the real components.
use bevy::{prelude::*, state::app::StatesPlugin};
use engine_core::{
    automata::{
        channel::{
            Backend, ChannelAction, ChannelCommand, ChannelId, ChannelIds, ChannelPlugin,
            ChannelSet, ChannelSettings, Channels, SWATCHES, Status, StepClock,
        },
        lenia::{LeniaControlPlugin, LeniaSimulation},
    },
    events::{AutomataCommand, AutomatonAdded, AutomatonId, AutomatonRemoved},
    prelude::{AppState, AutomataRegistry, RuleRegistry},
};
use engine_render::command_executor::{CommandExecutorPlugin, channels};

fn app() -> (App, ChannelId, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ChannelPlugin, LeniaControlPlugin))
        .init_resource::<AutomataRegistry>()
        .add_event::<AutomatonRemoved>()
        .add_systems(
            Update,
            (
                channels::route.in_set(ChannelSet::Route),
                channels::publish.in_set(ChannelSet::Publish),
            ),
        );
    let id = app
        .world_mut()
        .resource_mut::<ChannelIds>()
        .allocate()
        .unwrap();
    let sim = LeniaSimulation {
        paused: true,
        ..default()
    };
    let owner = app
        .world_mut()
        .spawn((
            AutomatonId(7),
            id,
            ChannelSettings::new(60.0),
            StepClock::default(),
            sim,
        ))
        .id();
    app.update();
    (app, id, owner)
}

fn send(app: &mut App, id: ChannelId, action: ChannelAction) {
    app.world_mut().send_event(ChannelCommand { id, action });
    app.update();
}

#[test]
fn lenia_rows_publish_as_gpu_channels() {
    let (app, id, _) = app();
    let row = app.world().resource::<Channels>().get(id).cloned().unwrap();
    assert_eq!(
        (row.backend, row.status, row.kind.as_str()),
        (Backend::Gpu, Status::Paused, "lenia")
    );
    assert_eq!(row.settings.rate_hz, 60.0);
}

#[test]
fn lifecycle_actions_map_to_lenia_actions() {
    let (mut app, id, owner) = app();
    send(&mut app, id, ChannelAction::Step);
    assert_eq!(
        app.world()
            .get::<LeniaSimulation>(owner)
            .unwrap()
            .target_tick,
        1
    );
    send(&mut app, id, ChannelAction::Reset);
    let sim = app.world().get::<LeniaSimulation>(owner).unwrap();
    assert_eq!((sim.target_tick, sim.epoch), (0, 1));
    send(&mut app, id, ChannelAction::Pause(false));
    assert!(!app.world().get::<LeniaSimulation>(owner).unwrap().paused);
    send(&mut app, id, ChannelAction::SetRate(1.0));
    send(&mut app, id, ChannelAction::SetColour(SWATCHES[2]));
    let settings = app.world().get::<ChannelSettings>(owner).unwrap();
    assert_eq!((settings.rate_hz, settings.colour), (1.0, SWATCHES[2]));
}

#[test]
fn remove_emits_automaton_removed_and_releases_the_id() {
    let (mut app, id, _) = app();
    send(&mut app, id, ChannelAction::Remove);
    let events = app.world().resource::<Events<AutomatonRemoved>>();
    let mut cursor = events.get_cursor();
    let ids: Vec<u32> = cursor.read(events).map(|e| e.id.0).collect();
    assert_eq!(ids, [7], "exactly one removal for the owner's automaton");
    assert_eq!(app.world().resource::<ChannelIds>().open(), 0);
}

fn executor_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .init_resource::<AutomataRegistry>()
        .init_resource::<RuleRegistry>()
        .add_event::<AutomataCommand>()
        .add_event::<AutomatonAdded>()
        .add_event::<AutomatonRemoved>()
        .add_plugins((
            models::automata::dynamical::lenia::plugin::LeniaPlugin,
            LeniaControlPlugin,
            CommandExecutorPlugin,
        ));
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::InGame);
    app.update();
    app
}

fn lenia_count(app: &mut App) -> usize {
    app.world_mut()
        .query::<&LeniaSimulation>()
        .iter(app.world())
        .count()
}

#[test]
fn lenia_spawn_allocates_a_selected_channel() {
    let mut app = executor_app();
    app.world_mut()
        .send_event(AutomataCommand::SeedPattern { id: "lenia".into() });
    app.update();
    let channels = app.world().resource::<Channels>();
    assert_eq!(channels.rows.len(), 1);
    assert_eq!(channels.selected, Some(channels.rows[0].id));
    app.world_mut().send_event(AutomataCommand::Clear);
    app.update();
    assert_eq!(app.world().resource::<ChannelIds>().open(), 0);
}

#[test]
fn lenia_spawn_refused_when_full() {
    let mut app = executor_app();
    for _ in 0..ChannelIds::CAPACITY {
        app.world_mut().resource_mut::<ChannelIds>().allocate();
    }
    app.world_mut()
        .send_event(AutomataCommand::SeedPattern { id: "lenia".into() });
    app.update();
    assert_eq!(lenia_count(&mut app), 0);
    assert!(app.world().resource::<AutomataRegistry>().list().is_empty());
}
