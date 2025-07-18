use bevy::prelude::*;
use crate::app::schedule::MainSet;

/// Collect raw device / network input and turn it into ECS events.
///
/// *Real* logic will arrive later; for now the system just logs a trace so
/// we can confirm ordering in the console.
fn collect_input() {
    // A real implementation would read `Input<KeyCode>` / sockets, then:
    //   * emit `CellToggleEvent`, `SpawnPatternEvent`, etc.
    //   * or mutate some `InputState` resource.
    trace!("<< Input stage >>");
}

/// Master plugin for the *Input* domain.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collect_input.in_set(MainSet::Input));
    }
}
