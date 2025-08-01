//! Atlas-image helper. © 2025 Obaven Inc. — Apache-2.0 OR MIT

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const MAX_W: u32 = 1024;
pub const MAX_H: u32 = 1024;
pub const MAX_LAYERS: u32 = 1;

/// Returns a cleared **R8Uint** 3-D image.
pub fn make_image(label: &'static str) -> Image {
    let data = vec![0u8; (MAX_W * MAX_H * MAX_LAYERS) as usize];

    let mut img = Image::new_fill(
        Extent3d { width: MAX_W, height: MAX_H, depth_or_array_layers: MAX_LAYERS },
        TextureDimension::D3,
        &data,
        TextureFormat::R8Uint,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.texture_descriptor.label = Some(label);
    img
}
