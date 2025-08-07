//! engine-core ▸ **automata**
//!
//! This module defines the runtime metadata that represents a *single
//! automaton* inside the ECS world as well as the GPU‐side atlas slice
//! descriptor that both the **main** and **render** worlds share.
//!
//! The GPU slice is now a proper *Bevy Component* so it can be attached
//! directly to the entity that owns the automaton.  Having it as a
//! component lets the render‐app query it without any cross-world
//! plumbing.
//! engine-core ▸ automata – runtime metadata shared with the GPU.
//! Now 3-D-aware: every slice knows its voxel **depth** inside the layer.

use std::sync::Arc;
use bevy::prelude::{Color, Component};
use glam::{UVec2, Vec3};
use serde_json::Value;
use simulation_kernel::{AutomatonRule, core::dim::Dim};

use crate::events::AutomatonId;

/* ------------------------------------------------------------------ */
/* GPU atlas slice                                                    */
/* ------------------------------------------------------------------ */

/// Identifies *where* an automaton’s state lives in the global
/// **3-D `R8Uint` atlas** used by the compute shaders.
///
/// New in the voxel build:
/// * **`depth`** – number of Z-slices the board occupies.
///   (For legacy 2-D rules this is always `1`.)
#[derive(Debug, Clone, Component)]
pub struct GpuGridSlice {
    /// Z-layer inside the atlas where the slice starts.
    pub layer: u32,
    /// Top-left corner — **in atlas texels**.
    pub offset: UVec2,
    /// Width × height — **in atlas texels**.
    pub size: UVec2,
    /// Depth of the slice in texels (≥ 1).
    pub depth: u32,
    /* rule info ---------------------------------------------------- */
    /// Rule Description
    pub rule: String,
    /// Rule Bit Format
    pub rule_bits: u32,
}

/* ------------------------------------------------------------------ */
/* Automaton registry entry                                           */
/* ------------------------------------------------------------------ */

/// In-memory description of a running automaton.
pub struct AutomatonInfo {
    /// Logical identifier assigned by the CPU registry.
    pub id: AutomatonId,
    /// Human‑readable name of the automaton/rule.
    pub name: String,
    /// Rule implementation driving the simulation.
    pub rule: Arc<dyn AutomatonRule<D = Dim> + Send + Sync>,
    /// Arbitrary JSON parameters passed to the rule.
    pub params: Value,
    /// Slice inside the global GPU atlas where state is stored.
    pub slice: GpuGridSlice,
    /// Dimensionality of the automaton.
    /// Size of one voxel edge in world units.
    pub voxel_size: f32,
    /// Translation applied to position the automaton in world space.
    pub world_offset: Vec3,
    /// Clear colour used for empty space.
    pub background_color: Color,
    /// Optional colour palette for visualisation.
    pub palette: Option<Vec<Color>>,
}

impl std::fmt::Debug for AutomatonInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutomatonInfo")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("slice", &self.slice)
            .field("voxel_size", &self.voxel_size)
            .finish()
    }
}
