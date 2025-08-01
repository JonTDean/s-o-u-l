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

use std::sync::Arc;

use bevy::prelude::{Color, Component};
use glam::{UVec2, Vec3};
use serde_json::Value;
use simulation_kernel::{AutomatonRule, core::dim::Dim};

use crate::events::AutomatonId;

/* ------------------------------------------------------------------ */
/* GPU atlas slice (shared GPU)                                 */
/* ------------------------------------------------------------------ */

/// Identifies *where* an automaton’s state lives inside the global
/// **3-D R8Uint atlas texture** used by the compute shaders.
///
/// The struct is lightweight (five u32s) and marked as a [`Component`]
/// so we can stick it on the ECS entity itself.  This keeps the data in
/// sync between the main and render worlds with zero bookkeeping.
#[derive(Debug, Clone, Component)]
pub struct GpuGridSlice {
    /// Z-layer inside the 3-D atlas.
    pub layer: u32,
    /// Top-left corner - **in atlas pixels**.
    pub offset: UVec2,
    /// Width × height - **in atlas pixels**.
    pub size: UVec2,
    /// Rule identifier (“life:conway”, “lenia:orbium”, …).
    pub rule: String,
    /// Packed Conway/Life/etc. flags honoured by the compute shader.
    pub rule_bits: u32,
}

/* ------------------------------------------------------------------ */
/* Automaton registry entry                                           */
/* ------------------------------------------------------------------ */

/// In-memory description of a running automaton.
///
/// One such struct exists per automaton and is stored inside
/// [`AutomataRegistry`].  It’s *not* a component because UI code needs
/// to access it even when the automaton entity might not be in scope.
pub struct AutomatonInfo {
    pub id:               AutomatonId,
    pub name:             String,
    pub rule:             Arc<dyn AutomatonRule<D = Dim> + Send + Sync>,
    pub params:           Value,
    pub seed_fn:          Option<fn(&mut ())>, // placeholder until CPU grid returns
    pub slice:            GpuGridSlice,
    pub dimension:        u8,                  // kept for backwards-compat with old UI
    pub voxel_size:       f32,
    pub world_offset:     Vec3,
    pub background_color: Color,
    pub palette:          Option<Vec<Color>>,
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
