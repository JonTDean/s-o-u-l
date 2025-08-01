//! seed/debug/square.rs
//! -------------------------------------------------------------
//! GPU **debug helper** – puts a 3 × 3 marker into the ping texture
//! so that voxel-atlas coordinates can be verified visually.
//!
//! * Call is fully **non-blocking**: the data upload is merely
//!   queued on the GPU command stream via [`RenderQueue::write_texture`].
//! * All arguments implement `Send + Sync`; the function is safe to
//!   invoke from any Bevy *Render* schedule system.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
#![cfg(feature = "gpu-debug")]

use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssets,
    renderer::RenderQueue as Queue,
    // `ImageDataLayout` was renamed in Bevy 0.16 → use the new type.
    render_resource::{Extent3d, Origin3d, TexelCopyBufferLayout},
    texture::GpuImage,
};

use crate::{graph::GpuGridSlice, plugin::GlobalStateTextures};

/// Write a solid 3 × 3 square filled with `value` into the *centre* of
/// `slice` inside the **ping texture**.
///
/// * `queue`      – thread-safe handle to the wgpu queue.  
/// * `gpu_images` – render-world view of allocated textures.  
/// * `textures`   – handles to the ping/pong/state images shared globally.  
/// * `slice`      – atlas location that receives the marker.  
/// * `value`      – single-channel byte (0–255) written to an `R8Uint` image.
///
/// ### Alignment
/// `TexelCopyBufferLayout.bytes_per_row` must be a multiple of the pixel
/// size (1 byte for `R8Uint`). For this tiny 3 × 3 upload, 3 bytes per row
/// is valid on all native back-ends. Do **not** use `RenderDevice::align_copy_bytes_per_row`
/// here – it would waste bandwidth for such a small debug write.
#[allow(clippy::too_many_arguments)]
pub fn seed_debug_square(
    queue:      &Queue,
    gpu_images: &RenderAssets<GpuImage>,
    textures:   &GlobalStateTextures,
    slice:      &GpuGridSlice,
    value:      u8,
) {
    /* 0 ─ Resolve GPU texture ---------------------------------------------- */
    let Some(ping_gpu) = gpu_images.get(&textures.ping) else {
        warn!("seed_debug_square: ping texture not available yet – skipped");
        return;
    };

    /* 1 ─ 3 × 3 payload ----------------------------------------------------- */
    let data = [value; 9];

    /* 2 ─ Destination description ------------------------------------------ */
    // Convenience helper returns a fully-populated TexelCopyTextureInfo.
    let mut dst = ping_gpu.texture.as_image_copy();
    // Centre the marker inside the atlas slice.
    let cx = slice.offset.x + slice.size.x / 2 - 1;
    let cy = slice.offset.y + slice.size.y / 2 - 1;
    dst.origin = Origin3d {
        x: cx,
        y: cy,
        z: slice.layer,
    };

    /* 3 ─ Queue the upload -------------------------------------------------- */
    queue.write_texture(
        dst,
        &data,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(3),
            rows_per_image: Some(3),
        },
        Extent3d {
            width: 3,
            height: 3,
            depth_or_array_layers: 1,
        },
    );
}
