//! Shared structs & resources for the HUD minimap.
//!
//! The *main‑world* owns the **logical** map (`MinimapTextures`) that
//! maps each [`AutomatonId`] to a lazily‑initialised [`Image`] handle.
//! The **render‑world** fills in the handle with a 2‑D view into the 3‑D
//! voxel atlas (see `engine‑gpu::graph::minimap_views`).  UI code can
//! therefore treat the minimap exactly like any other Bevy texture – no
//! staging copies, no extra VRAM.

use std::collections::HashMap;

use bevy::prelude::*;

use engine_core::{
    automata::GpuGridSlice,
    events::AutomatonId,
};

/// Per‑automaton data required by the minimap panel.
#[derive(Clone)]
pub struct MinimapEntry {
    pub slice:   GpuGridSlice,   // where in the atlas this board lives
    pub texture: Handle<Image>,  // 2‑D view (filled in render‑world)
}

/// Global mapping: `AutomatonId → minimap thumbnail`.
#[derive(Resource, Default)]
pub struct MinimapTextures(pub HashMap<AutomatonId, MinimapEntry>);

