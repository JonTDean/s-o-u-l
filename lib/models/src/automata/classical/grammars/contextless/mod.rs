//! Type‑2 **Context‑Free** automata (push‑down, L‑systems …).

use bevy::prelude::*;

/// Public plugin exported by the parent module.
pub struct ContextFreeAutomataPlugin;

impl Plugin for ContextFreeAutomataPlugin {
    fn build(&self, app: &mut App) {
        use crate::workbench::catalog::{Kind, register};
        register(app, &[Kind::Pushdown, Kind::Parentheses, Kind::Koch]);
    }
}

pub mod l_system;
pub mod ll1_pushdown;
