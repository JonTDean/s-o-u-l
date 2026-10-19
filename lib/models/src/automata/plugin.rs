//! Thin umbrella plugin that simply wires together the sub‑families
//! (classical, dynamical, …).  All heavy lifting happens inside those
//! specialised sub‑plugins.
// computational_intelligence/src/automata/plugin.rs
use super::classical::plugin::ClassicalAutomataPlugin;
use super::dynamical::plugin::DynamicalAutomataPlugin;
use bevy::prelude::*;

pub struct AutomataPlugin;
impl Plugin for AutomataPlugin {
    fn build(&self, app: &mut App) {
        // Always register rule sets (classical + dynamical)
        app.add_plugins((
            ClassicalAutomataPlugin,
            DynamicalAutomataPlugin,
            crate::workbench::plugin::WorkbenchPlugin,
        ));
        // Command execution belongs to engine-render; models register rule families.
    }
}
