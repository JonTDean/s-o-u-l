//! Automata grid–rendering material (Bevy 0.16.1).
//!
//! This `Material2d` implementation draws a huge cellular‑automata grid on a
//! single *world‑aligned* quad.  It is designed for multi‑threaded Bevy apps:
//! * all data is `Send + Sync`,
//! * the GPU bindings are generated automatically via `AsBindGroup`,
//! * only the POSITION vertex attribute is streamed, minimising bandwidth.
//!
//! ### Bevy 0.16 migration notes
//! * `TypeUuid` **was removed** by Asset V2 – assets now derive `TypePath`
//!   instead (see <https://github.com/bevyengine/bevy/issues/9714>).
//! * `#[uuid = "..."]` attributes are therefore illegal; stable identity comes
//!   from the type path string supplied by `TypePath`.

use bevy::{prelude::*, sprite::AlphaMode2d};
use bevy::asset::Asset;
use bevy_reflect::TypePath;
use bevy::render::{
    mesh::{Mesh, MeshVertexBufferLayoutRef},
    render_resource::{AsBindGroup, ShaderRef, ShaderType, SpecializedMeshPipelineError},
};
use bevy::sprite::{Material2d, Material2dKey};

/* ───────────────────────────── Material asset ───────────────────────────── */

/// GPU material for visualising a 2‑D automata grid stored in a texture.
///
/// *Derives*:  
/// * `Asset`   – allows storage in `Assets<Self>` and hot‑reloading.  
/// * `AsBindGroup` – generates the WGSL bind‑group layout automatically.
/// * `Clone`  – required by `Material2d`.
/// * `TypePath` – stable identifier required by Asset V2 :contentReference[oaicite:6]{index=6}.  
#[derive(Asset, AsBindGroup, Clone, TypePath)]
pub struct AutomataMaterial {
    #[uniform(0)] pub params: AutomataParams,

    #[texture(1)] 
    #[sampler(2)] 
    pub grid_texture: Handle<Image>,
}

/* ───────────────────────────── Uniform block ────────────────────────────── */

/// CPU <‑> GPU uniform layout (std140).
#[repr(C)]
#[derive(Debug, Copy, Clone, ShaderType)]
pub struct AutomataParams {
    /// World‑space camera centre.
    pub camera_pos:   Vec2,
    /// Zoom multiplier (world units per screen unit).
    pub zoom:         f32,
    /// Width/height of one automata cell in world units.
    pub cell_size:    f32,
    /// Size of the grid texture in texels.
    pub texture_size: Vec2,
    /// RGBA colour for a dead cell.
    pub dead_color:   Vec4,
    /// RGBA colour for a live cell.
    pub alive_color:  Vec4,
}

/* ─────────────────────────── Material2d impl ────────────────────────────── */

impl Material2d for AutomataMaterial {
    /// Both stages are in one WGSL file for simplicity.
    fn vertex_shader() -> ShaderRef {
        "shaders/automata_material.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/automata_material.wgsl".into()
    }

    /// Provide a *minimal* vertex layout: POSITION only (location 0).
    /// Note the `.0` access – `MeshVertexBufferLayoutRef` is an interned wrapper
    /// in Bevy 0.14+ :contentReference[oaicite:7]{index=7}.
    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout:     &MeshVertexBufferLayoutRef,
        _key:       Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
    
    // NEW override – use the *2‑D* alpha‑mode enum
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend          // ← rendered in the transparent pass
    }
}
