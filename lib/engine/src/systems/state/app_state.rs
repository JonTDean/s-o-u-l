use bevy::prelude::*;

/// Defines the high-level states of the application (main menu, in-game, etc.).
#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum AppState {
    /// Main menu state (initial state).
    #[default]
    MainMenu,
    /// State for configuring a new scenario.
    NewScenario,
    /// State for loading a saved scenario.
    LoadScenario,
    /// State for the options/settings screen.
    Options,
    /// State for when a simulation is running (in-game).
    InGame,
    /// State for when the simulation is paused.
    Paused,
}
