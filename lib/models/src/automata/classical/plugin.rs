use bevy::prelude::*;

use crate::automata::classical::{contextful, contextless, regular, turing};

/// Aggregates every classical family into one Bevy plugin.
pub struct ClassicalAutomataPlugin;
impl Plugin for ClassicalAutomataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            regular::RegularAutomataPlugin,
            contextless::ContextFreeAutomataPlugin,
            contextful::ContextSensitiveAutomataPlugin,
            turing::TuringAutomataPlugin,
        ));
    }
}
