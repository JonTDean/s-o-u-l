//! render/minimap.rs – runtime mapping “AutomatonId → minimap texture”.
//!
//! This is a verbatim lift-&-shift of the old
//! `render::worldgrid::minimap` module so downstream UI code can keep the
//! exact same data-flow with **zero logic changes**.

use std::collections::HashMap;
use bevy::prelude::*;
use engine_core::{
    automata::GpuGridSlice,
    events::AutomatonId,
};

/// Per-automaton data required by the minimap overlay.
#[derive(Clone)]
pub struct MinimapEntry {
    pub slice:   GpuGridSlice,  // where in the atlas this board lives
    pub texture: Handle<Image>, // 2-D view (filled in render-world)
}

/// Global mapping: `AutomatonId → minimap thumbnail`.
#[derive(Resource, Default)]
pub struct MinimapTextures(pub HashMap<AutomatonId, MinimapEntry>);