//! Type‑1 **Context‑Sensitive** automata (linear‑bounded automata).

use bevy::prelude::*;

pub struct ContextSensitiveAutomataPlugin;

impl Plugin for ContextSensitiveAutomataPlugin {
    fn build(&self, app: &mut App) {
        use crate::workbench::catalog::{Kind, register};
        register(app, &[Kind::Lba]);
    }
}

pub mod lba_examples;
