//! Channel-contract routing, id reconciliation and row publishing for CPU sessions.
use crate::workbench::{
    experiment::config::Settings,
    plugin::{FULL_NOTE, Workbench},
    session::Session,
};
use bevy::prelude::*;
use engine_core::{
    automata::channel::{
        Backend, ChannelAction, ChannelCommand, ChannelId, ChannelIds, ChannelRow, ChannelSettings,
        Channels,
    },
    prelude::AppState,
};
pub mod metrics;

/// Replace a session's experiment settings; refusals become the session note.
#[derive(Event, Clone, Debug)]
pub struct ConfigureSession {
    /// Target channel.
    pub id: ChannelId,
    /// Candidate settings, validated by `Session::configure`.
    pub settings: Box<Settings>,
    /// Apply and reset (required for seed and population changes).
    pub reset: bool,
}

/// Give unassigned sessions (new spawns, loaded projects) ids and release vanished ones.
///
/// Project load replaces `Workbench::sessions` wholesale, so ownership is
/// reconciled each frame instead of being tracked at every mutation site.
pub(in crate::workbench) fn reconcile(
    mut bench: ResMut<Workbench>,
    mut ids: ResMut<ChannelIds>,
    mut channels: ResMut<Channels>,
    mut known: Local<Vec<ChannelId>>,
) {
    for id in known.iter() {
        if !bench.sessions.iter().any(|s| s.channel == *id) {
            ids.release(*id);
        }
    }
    let (mut adopted, mut dropped) = (None, false);
    bench.sessions.retain_mut(|session| {
        if session.channel != ChannelId::UNASSIGNED {
            return true;
        }
        let Some(id) = ids.allocate() else {
            dropped = true;
            return false;
        };
        session.channel = id;
        adopted = Some(id);
        true
    });
    if dropped {
        bench.error = Some(FULL_NOTE.into());
    }
    if adopted.is_some() {
        channels.selected = adopted;
    }
    *known = bench.sessions.iter().map(|s| s.channel).collect();
}

/// Apply channel commands addressed to CPU sessions; other ids are skipped.
pub(in crate::workbench) fn route(
    mut commands: EventReader<ChannelCommand>,
    mut configure: EventReader<ConfigureSession>,
    mut bench: ResMut<Workbench>,
    mut ids: ResMut<ChannelIds>,
) {
    for command in commands.read() {
        let Some(index) = bench.sessions.iter().position(|s| s.channel == command.id) else {
            continue;
        };
        let session = &mut bench.sessions[index];
        match &command.action {
            ChannelAction::Pause(paused) if !session.halted() => session.paused = *paused,
            ChannelAction::Step => session.single_step(),
            ChannelAction::Paint {
                points,
                value,
                start,
            } => session.note = session.paint(points, *value, *start).err().map(Into::into),
            ChannelAction::UndoPaint(redo) => {
                session.note = session.undo_paint(*redo).err().map(Into::into)
            }
            ChannelAction::Reset => session.note = session.reset().err().map(Into::into),
            ChannelAction::SetRate(rate) => session.settings.rate_hz = rate.clamp(1.0, 60.0),
            ChannelAction::SetColour(colour) => session.settings.colour = *colour,
            ChannelAction::SetInput(text) => session.input = text.clone(),
            ChannelAction::Remove => {
                // Release now so a spawn in the very next frame sees the capacity.
                ids.release(bench.sessions.remove(index).channel);
                bench.selected = bench.selected.min(bench.sessions.len().saturating_sub(1));
            }
            _ => {}
        }
    }
    for request in configure.read() {
        if let Some(session) = bench.sessions.iter_mut().find(|s| s.channel == request.id) {
            let result = session.configure((*request.settings).clone(), request.reset);
            session.note = result.err().map(Into::into);
        }
    }
}

/// Advance each running session on its own rate clock.
pub(in crate::workbench) fn tick(
    mut bench: ResMut<Workbench>,
    time: Option<Res<Time>>,
    state: Option<Res<State<AppState>>>,
) {
    if state.is_some_and(|s| *s.get() != AppState::InGame) {
        return;
    }
    let seconds = time.map_or(0.0, |time| time.delta_secs());
    for session in &mut bench.sessions {
        session.elapse(seconds);
    }
}

/// Replace the CPU rows of the channel snapshot.
pub(in crate::workbench) fn publish(bench: Res<Workbench>, mut channels: ResMut<Channels>) {
    channels.publish(Backend::Cpu, bench.sessions.iter().map(row));
}

fn row(session: &Session) -> ChannelRow {
    ChannelRow {
        id: session.channel,
        kind: session.kind.id().into(),
        backend: Backend::Cpu,
        status: session.status(),
        tick: session.ticks,
        epoch: session.epoch,
        settings: ChannelSettings {
            rate_hz: session.settings.rate_hz,
            colour: session.settings.colour,
        },
    }
}
#[cfg(test)]
mod tests;
