//! Native command routing and fixed-rate CPU sessions for the family workbench.
use super::{
    catalog::Kind,
    operations::channels::{self, ConfigureSession},
    session::Session,
};
use bevy::prelude::*;
use engine_core::{
    automata::channel::{ChannelId, ChannelIds, ChannelSet, ensure_plugin},
    events::AutomataCommand,
    prelude::AppState,
};

/// At most eight isolated sessions; selection affects presentation only.
#[derive(Resource, Default)]
pub struct Workbench {
    pub sessions: Vec<Session>,
    pub selected: usize,
    pub error: Option<String>,
}
impl Workbench {
    /// Add a paused instance unless the eight-session capacity is exhausted.
    pub fn spawn(&mut self, kind: Kind) {
        if self.sessions.len() >= 8 {
            self.error =
                Some("Eight sessions are open. Remove one or clear all before spawning.".into());
            return;
        }
        self.sessions.push(Session::new(kind));
        self.selected = self.sessions.len() - 1;
        self.error = None;
    }
    /// Release all instances, selection, errors and accumulated wall time.
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.selected = 0;
        self.error = None;
    }
}

/// Shown when the shared eight-channel capacity is exhausted.
pub const FULL_NOTE: &str = "8 of 8 open — close one to add another";

/// Installs spawn/clear routing, channel commands and per-session rate clocks.
pub struct WorkbenchPlugin;
impl Plugin for WorkbenchPlugin {
    fn build(&self, app: &mut App) {
        ensure_plugin(app);
        app.init_resource::<Workbench>()
            .add_event::<ConfigureSession>()
            .add_systems(
                Update,
                (
                    (lifecycle, channels::reconcile, channels::route)
                        .chain()
                        .in_set(ChannelSet::Route),
                    channels::tick.in_set(ChannelSet::Tick),
                    (channels::publish, channels::metrics::publish).in_set(ChannelSet::Publish),
                ),
            );
    }
}
fn lifecycle(
    mut commands: EventReader<AutomataCommand>,
    mut bench: ResMut<Workbench>,
    ids: Res<ChannelIds>,
    state: Option<Res<State<AppState>>>,
) {
    if state
        .as_ref()
        .is_some_and(|s| *s.get() == AppState::MainMenu)
    {
        commands.clear();
        bench.clear();
        return;
    }
    for command in commands.read() {
        match command {
            AutomataCommand::Clear => bench.clear(),
            AutomataCommand::RestoreLenia(_) => {}
            AutomataCommand::SeedPattern { id } => {
                let Some(kind) = Kind::parse(id) else {
                    continue;
                };
                // Capacity is shared with GPU channels; unadopted spawns count too.
                let pending = bench
                    .sessions
                    .iter()
                    .filter(|s| s.channel == ChannelId::UNASSIGNED)
                    .count();
                if ids.open() + pending >= ChannelIds::CAPACITY {
                    bench.error = Some(FULL_NOTE.into());
                } else {
                    bench.spawn(kind);
                }
            }
        }
    }
}
