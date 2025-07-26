//! Thin umbrella plugin that simply wires together the sub‑families
//! (classical, dynamical, …).  All heavy lifting happens inside those
//! specialised sub‑plugins.

use bevy::prelude::*;

pub struct AutomataPlugin;

impl Plugin for AutomataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            super::classical::plugin::ClassicalAutomataPlugin,
            super::dynamical::plugin::DynamicalAutomataPlugin,
            crate::bridges::world_stepper::WorldStepperPlugin,
            crate::bridges::command_executor::CommandExecutorPlugin,
        ));
    }
}
