//! Deterministic volumetric fixture for the portable renderer preview.
use crate::plugin::GlobalVoxelAtlas;
use bevy::render::{
    render_asset::RenderAssets,
    render_resource::{Extent3d, Origin3d, TexelCopyBufferLayout},
    renderer::RenderQueue,
    texture::GpuImage,
};
use engine_core::automata::GpuGridSlice;

pub(crate) fn fixture() -> Vec<u8> {
    let mut cells = vec![0; 16 * 16 * 16];
    for z in 2..14 {
        for y in 1..13 {
            for x in 2..14 {
                let platform = y <= 2;
                let pillar = (x <= 4 || x >= 11) && (4..=11).contains(&z) && y <= 10;
                let bridge = y >= 9 && (4..=11).contains(&z);
                if platform || pillar || bridge {
                    cells[z * 256 + y * 16 + x] = 255;
                }
            }
        }
    }
    cells
}

pub(crate) fn upload(
    queue: &RenderQueue,
    images: &RenderAssets<GpuImage>,
    atlas: &GlobalVoxelAtlas,
    slice: &GpuGridSlice,
) {
    let Some(image) = images.get(&atlas.atlas) else {
        return;
    };
    let mut destination = image.texture.as_image_copy();
    destination.origin = Origin3d {
        x: slice.offset.x,
        y: slice.offset.y,
        z: slice.layer,
    };
    queue.write_texture(
        destination,
        &fixture(),
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(16),
            rows_per_image: Some(16),
        },
        Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 16,
        },
    );
}
