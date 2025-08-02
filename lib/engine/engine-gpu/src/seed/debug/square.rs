//! GPU debug helper – writes a 3 × 3 marker into the atlas so voxel
//! coordinates can be verified visually.
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
#![cfg(feature = "gpu-debug")]

use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssets,
    renderer::RenderQueue as Queue,
    render_resource::{Extent3d, Origin3d, TexelCopyBufferLayout},
    texture::GpuImage,
};

use crate::{graph::GpuGridSlice, plugin::GlobalVoxelAtlas};

/// Write a solid 3 × 3 square (`value`) into the *centre* of `slice`.
#[allow(clippy::too_many_arguments)]
pub fn seed_debug_square(
    queue:      &Queue,
    gpu_images: &RenderAssets<GpuImage>,
    textures:   &GlobalVoxelAtlas,
    slice:      &GpuGridSlice,
    value:      u8,
) {
    // 0 ░ resolve texture
    let Some(atlas_gpu) = gpu_images.get(&textures.atlas) else {
        warn!("seed_debug_square: atlas not ready – skipped");
        return;
    };

    // 1 ░ payload
    let data = [value; 9];

    // 2 ░ destination
    let mut dst = atlas_gpu.texture.as_image_copy();
    let cx = slice.offset.x + slice.size.x / 2 - 1;
    let cy = slice.offset.y + slice.size.y / 2 - 1;
    dst.origin = Origin3d { x: cx, y: cy, z: slice.layer };

    // 3 ░ upload three consecutive layers
    for dz in 0..3 {
        dst.origin.z = slice.layer + dz;
        queue.write_texture(
            dst,
            &data,
            TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(3), rows_per_image: Some(3) },
            Extent3d { width: 3, height: 3, depth_or_array_layers: 1 },
        );
    }
}
