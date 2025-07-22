// input/plugin.rs
use bevy::prelude::*;
use engine_core::schedule::MainSet;

use crate::controls::camera_control::CameraControlPlugin;

fn collect_input() {
    trace!("<< Input stage >>");
    // In the future, read Input<KeyCode> or other input resources and emit events.
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Register the input collection system in the Input stage of the Update schedule
        app.add_systems(Update, collect_input.in_set(MainSet::Input));
        app.add_plugins(CameraControlPlugin); 

    }
}
