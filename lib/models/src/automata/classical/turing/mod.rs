//! Type‑0 **Turing‑complete** automata.

use bevy::prelude::*;

pub struct TuringAutomataPlugin;

impl Plugin for TuringAutomataPlugin {
    fn build(&self, app: &mut App) {
        use crate::workbench::catalog::{Kind, register};
        register(app, &[Kind::Turing]);
    }
}

pub mod universal_tm;
