use bevy::prelude::*;

use crate::events::AutomataCommand;

pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<AutomataCommand>();
    }
}