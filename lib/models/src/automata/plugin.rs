//! Thin umbrella plugin that simply wires together the sub‑families
//! (classical, dynamical, …).  All heavy lifting happens inside those
//! specialised sub‑plugins.
// computational_intelligence/src/automata/plugin.rs
use bevy::prelude::*;
use super::classical::plugin::ClassicalAutomataPlugin;
use super::dynamical::plugin::DynamicalAutomataPlugin;
use engine_render::command_executor::CommandExecutorPlugin;

pub struct AutomataPlugin;
impl Plugin for AutomataPlugin {
    fn build(&self, app: &mut App) {
        // Always register rule sets (classical + dynamical)
        app.add_plugins((ClassicalAutomataPlugin, DynamicalAutomataPlugin));
        // Only add CPU world-stepper if GPU compute is not active
        
        // Always add the command executor for spawning/clearing automata
        app.add_plugins(CommandExecutorPlugin);
    }
}
