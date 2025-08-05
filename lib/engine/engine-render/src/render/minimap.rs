//! render/minimap.rs – runtime mapping “AutomatonId → minimap texture”.
//!
//! This is a verbatim lift-&-shift of the old
//! `render::worldgrid::minimap` module so downstream UI code can keep the
//! exact same data-flow with **zero logic changes**.

use bevy::prelude::*;
use engine_core::{automata::GpuGridSlice, events::AutomatonId};
use std::collections::HashMap;

/// Per-automaton data required by the minimap overlay.
#[derive(Clone)]
pub struct MinimapEntry {
    /// Atlas slice describing where the board lives.
    pub slice: GpuGridSlice,
    /// 2‑D texture view populated in the render world.
    pub texture: Handle<Image>,
}

/// Global mapping: `AutomatonId → minimap thumbnail`.
#[derive(Resource, Default)]
pub struct MinimapTextures(pub HashMap<AutomatonId, MinimapEntry>);
