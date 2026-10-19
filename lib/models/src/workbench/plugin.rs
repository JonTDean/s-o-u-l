//! Native command routing and fixed-rate CPU sessions for the family workbench.
use super::{catalog::Kind, session::Session};
use bevy::prelude::*;
use engine_core::{events::AutomataCommand, prelude::AppState};

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

/// Installs production spawn/clear routing and the ten-hertz CPU update loop.
pub struct WorkbenchPlugin;
impl Plugin for WorkbenchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Workbench>().add_systems(Update, update);
    }
}
fn update(
    mut commands: EventReader<AutomataCommand>,
    mut bench: ResMut<Workbench>,
    time: Option<Res<Time>>,
    state: Option<Res<State<AppState>>>,
) {
    if state
        .as_ref()
        .is_some_and(|s| *s.get() == AppState::MainMenu)
    {
        bench.clear();
        return;
    }
    for command in commands.read() {
        match command {
            AutomataCommand::Clear => bench.clear(),
            AutomataCommand::SeedPattern { id } => {
                if let Some(kind) = Kind::parse(id) {
                    bench.spawn(kind);
                }
            }
        }
    }
    if state.as_ref().is_some_and(|s| *s.get() != AppState::InGame) {
        return;
    }
    let seconds = time.map_or(0.0, |time| time.delta_secs());
    for session in &mut bench.sessions {
        session.elapse(seconds);
    }
}
