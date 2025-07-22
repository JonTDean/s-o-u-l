//! Root plugin of the *computational‑intelligence* crate.
//!
//! Right now it only forwards to the automata subtree, but keeping a
//! dedicated root plugin lets us add analytics‑, ML‑ or optimiser
//! sub‑modules later without touching the main game.

use bevy::prelude::*;

use crate::automata::plugin::AutomataPlugin;

pub struct ComputationalIntelligencePlugin;
impl Plugin for ComputationalIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AutomataPlugin); 
    }
}
