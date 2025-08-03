//! Diagnostics & FPS counters (stub).
#![allow(dead_code)]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

pub struct MonitoringPlugin;

impl Plugin for MonitoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
        ));
    }
}
