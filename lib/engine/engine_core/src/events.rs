use bevy::prelude::*;

#[derive(Event)]
pub enum AutomataCommand {
    SeedPattern { id: String },
    Clear,
}