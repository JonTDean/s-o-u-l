//! Atlas-image helpers.
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};

/// Width of the voxel atlas in texels.
pub const MAX_W: u32 = 1024;
/// Height of the voxel atlas in texels.
pub const MAX_H: u32 = 1024;
/// Maximum number of atlas layers.
pub const MAX_LAYERS: u32 = 128;

/* ────────────────────────────────────────────────────────────── */
/* 3-D sparse voxel atlas (R8Uint, storage-qualified)            */
/* ────────────────────────────────────────────────────────────── */
/// Create a 3‑D R8Uint texture suitable for storing automaton state.
pub fn make_atlas(label: &'static str) -> Image {
    let data = vec![0u8; (MAX_W * MAX_H * MAX_LAYERS) as usize];

    let mut img = Image::new_fill(
        Extent3d {
            width: MAX_W,
            height: MAX_H,
            depth_or_array_layers: MAX_LAYERS,
        },
        TextureDimension::D3,
        &data,
        TextureFormat::R8Uint,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.texture_descriptor.label = Some(label);
    img.texture_descriptor.usage |= TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    img
}

/* ────────────────────────────────────────────────────────────── */
/* 2-D signalling texture (one byte per texel, same footprint    */
/* as a single atlas layer).                                     */
/* ────────────────────────────────────────────────────────────── */
/// Create a 2‑D signalling texture sharing the atlas footprint.
pub fn make_image(label: &'static str) -> Image {
    let data = vec![0u8; (MAX_W * MAX_H) as usize];

    let mut img = Image::new_fill(
        Extent3d {
            width: MAX_W,
            height: MAX_H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::R8Uint,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.texture_descriptor.label = Some(label);
    img.texture_descriptor.usage |= TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    img
}
