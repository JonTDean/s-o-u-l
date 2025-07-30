//! Type‑0 **Turing‑complete** automata.

use bevy::prelude::*;

pub struct TuringAutomataPlugin;

impl Plugin for TuringAutomataPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register Universal TM, tag‑systems, etc.
    }
}

pub mod universal_tm;
