//! Type‑2 **Context‑Free** automata (push‑down, L‑systems …).
//! Right now only the plugin skeleton exists.

use bevy::prelude::*;

/// Public plugin exported by the parent module.
pub struct ContextFreeAutomataPlugin;

impl Plugin for ContextFreeAutomataPlugin {
    fn build(&self, _app: &mut App) {
        // register rules here later
    }
}

/* ───── sub‑modules to be filled in ───── */
pub mod ll1_pushdown;
pub mod l_system;
