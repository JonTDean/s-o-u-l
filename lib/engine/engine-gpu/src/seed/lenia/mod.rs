//! Runtime seeding helpers for Lenia-family rules.
//!
//! Each helper writes a small density blob straight into the R8-atlas so the
//! very first simulation tick already has something to evolve.
//!
//! ────────────────────────────────────────────────────────────────
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//

use bevy::{
    render::{
        render_asset::RenderAssets,
        renderer::RenderQueue,
        render_resource::{Extent3d, Origin3d, TexelCopyBufferLayout},
        texture::GpuImage,
    },
};

use crate::{
    graph::RenderGpuGridSlice,
    plugin::GlobalVoxelAtlas,
};

/// 𝛕-shaped Orbium seed (7 × 7) expressed as raw R8 intensities.
///
/// The pattern is taken from the reference implementation in  
/// *“Lenia — Biology of Artificial Life”* (Chan 2019)
/// but axis-aligned so we can drop it in the atlas without rotation maths.
const ORBIUM_7X7: [u8; 49] = [
    0,  0,  32,  64,  32,  0,  0,
    0,  32,  64, 128,  64, 32,  0,
    32, 64, 128, 255, 128, 64, 32,
    64,128, 255, 255, 255,128, 64,
    32, 64, 128, 255, 128, 64, 32,
    0,  32,  64, 128,  64, 32,  0,
    0,   0,  32,  64,  32,  0,  0,
];

/// Upload the 7 × 7 Orbium kernel in the *centre* of `slice`.
pub fn seed_orbium(
    queue:      &RenderQueue,
    images:     &RenderAssets<GpuImage>,
    atlas:      &GlobalVoxelAtlas,
    slice:      &RenderGpuGridSlice,
) {
    let Some(tex) = images.get(&atlas.atlas) else { return };

    let cx = slice.0.offset.x + slice.0.size.x / 2 - 3;
    let cy = slice.0.offset.y + slice.0.size.y / 2 - 3;

    let mut dst = tex.texture.as_image_copy();
    dst.origin = Origin3d {
        x: cx,
        y: cy,
        z: slice.0.layer,
    };

    queue.write_texture(
        dst,
        &ORBIUM_7X7,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(7),
            rows_per_image: Some(7),
        },
        Extent3d {
            width: 7,
            height: 7,
            depth_or_array_layers: 1,
        },
    );
}
