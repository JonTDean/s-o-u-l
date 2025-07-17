use bevy::prelude::*;

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default] MainMenu,
    NewScenario,
    LoadScenario,
    Options,
    InGame,
}