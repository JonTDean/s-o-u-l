//! GPU‑side uniform / storage structs shared with WGSL.

use bevy::render::render_resource::*;
use bytemuck::{Pod, Zeroable};

/// Parameters for **one** automaton slice in the texture atlas.
///
/// * Layout ‑ std430 (because it is placed in a STORAGE buffer).  
/// * Matches the `AutomatonParams` struct in every WGSL rule shader.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct AutomatonParams {
    /// Width  of the automaton grid in texels.
    pub size_x: u32,
    /// Height of the automaton grid in texels.
    pub size_y: u32,
    /// Z‑layer inside the 3‑D atlas texture where this board lives.
    pub layer:  u32,
    /// Packed rule bits (e.g. Conway B/S or Lenia kernel index).
    pub rule:   u32,
}
