//! Channel routing, id allocation and row publishing for GPU-backed Lenia owners.
use bevy::{ecs::system::EntityCommands, prelude::*};
use engine_core::{
    automata::{
        channel::{
            Backend, ChannelAction, ChannelCommand, ChannelId, ChannelIds, ChannelRow,
            ChannelSettings, Channels, Status, StepClock,
        },
        lenia::{LeniaAction, LeniaCommand, LeniaSimulation},
    },
    events::{AutomatonId, AutomatonRemoved},
    prelude::AutomataRegistry,
};

/// Lenia keeps the fixed simulation rate it ran at before channels existed.
pub const LENIA_RATE_HZ: f32 = 60.0;

/// Attach channel components to a new owner and select it.
pub(super) fn adopt(owner: &mut EntityCommands, id: ChannelId, channels: &mut Channels) {
    owner.insert((
        id,
        ChannelSettings::new(LENIA_RATE_HZ),
        StepClock::default(),
    ));
    channels.selected = Some(id);
}

/// Translate channel commands into Lenia commands, settings changes and removals.
pub fn route(
    mut commands: EventReader<ChannelCommand>,
    mut owners: Query<
        (Entity, &ChannelId, &AutomatonId, &mut ChannelSettings),
        With<LeniaSimulation>,
    >,
    mut lenia: EventWriter<LeniaCommand>,
    mut registry: ResMut<AutomataRegistry>,
    mut removed: EventWriter<AutomatonRemoved>,
    mut ids: ResMut<ChannelIds>,
) {
    for command in commands.read() {
        let Some((entity, id, automaton, mut settings)) =
            owners.iter_mut().find(|(_, id, ..)| **id == command.id)
        else {
            continue;
        };
        let mut send = |action| {
            lenia.write(LeniaCommand { entity, action });
        };
        match &command.action {
            ChannelAction::Pause(paused) => send(LeniaAction::Pause(*paused)),
            ChannelAction::Step => send(LeniaAction::Step),
            ChannelAction::Reset => send(LeniaAction::Reset),
            ChannelAction::Lenia(action) => send(*action),
            ChannelAction::SetRate(rate) => settings.rate_hz = rate.clamp(1.0, 60.0),
            ChannelAction::SetColour(colour) => settings.colour = *colour,
            ChannelAction::Remove => {
                // The atlas allocator despawns the owner when it sees AutomatonRemoved.
                registry.remove(*automaton);
                removed.write(AutomatonRemoved { id: *automaton });
                ids.release(*id);
            }
            _ => {}
        }
    }
}

/// Replace the GPU rows of the channel snapshot.
pub fn publish(
    owners: Query<(&ChannelId, &LeniaSimulation, &ChannelSettings)>,
    mut channels: ResMut<Channels>,
) {
    channels.publish(
        Backend::Gpu,
        owners.iter().map(|(id, sim, settings)| ChannelRow {
            id: *id,
            kind: "lenia".into(),
            backend: Backend::Gpu,
            status: if sim.paused {
                Status::Paused
            } else {
                Status::Running
            },
            tick: sim.target_tick,
            epoch: sim.epoch,
            settings: *settings,
        }),
    );
}
