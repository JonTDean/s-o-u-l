//! Aggregates all *dynamical* automata: Lenia, HPP lattice‑gas, etc.

use bevy::prelude::*;

use crate::automata::dynamical::{lenia::plugin::LeniaPlugin, particle::plugin::ParticleAutomataPlugin};

/// Dynamical automata master plugin.
pub struct DynamicalAutomataPlugin;

impl Plugin for DynamicalAutomataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LeniaPlugin,
            ParticleAutomataPlugin,
        ));
    }
}
