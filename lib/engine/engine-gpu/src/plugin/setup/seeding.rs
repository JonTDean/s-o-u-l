//! Seed newly extracted slices only after their GPU texture is ready.
use crate::{graph::ExtractedGpuSlices, plugin::GlobalVoxelAtlas, seed::lenia::seed_orbium};
use bevy::{
    prelude::*,
    render::{render_asset::RenderAssets, renderer::RenderQueue, texture::GpuImage},
};
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(crate) struct SeededSlices(HashSet<(Entity, u32, u32, u32, String)>);

pub(in crate::plugin) fn seed_new_slices(
    slices: Res<ExtractedGpuSlices>,
    mut done: ResMut<SeededSlices>,
    queue: Res<RenderQueue>,
    images: Res<RenderAssets<GpuImage>>,
    atlas: Res<GlobalVoxelAtlas>,
) {
    if images.get(&atlas.atlas).is_none() {
        return;
    }
    let active: HashSet<_> = slices
        .0
        .iter()
        .map(|s| (s.1, s.0.layer, s.0.offset.x, s.0.offset.y, s.0.rule.clone()))
        .collect();
    done.0.retain(|key| active.contains(key));
    for slice in &slices.0 {
        let key = (
            slice.1,
            slice.0.layer,
            slice.0.offset.x,
            slice.0.offset.y,
            slice.0.rule.clone(),
        );
        if done.0.contains(&key) {
            continue;
        }
        match slice.0.rule.as_str() {
            "lenia" | "lenia:orbium" => seed_orbium(&queue, &images, &atlas, slice),
            "voxel-demo" => crate::seed::demo::upload(&queue, &images, &atlas, &slice.0),
            _ => {}
        }
        done.0.insert(key);
    }
}

#[cfg(feature = "gpu-debug")]
pub(in crate::plugin) fn apply_debug_seeds(
    mut pending: ResMut<crate::plugin::resources::PendingDebugSeeds>,
    queue: Res<RenderQueue>,
    images: Res<RenderAssets<GpuImage>>,
    atlas: Res<GlobalVoxelAtlas>,
) {
    if images.get(&atlas.atlas).is_none() {
        return;
    }
    for event in pending.0.drain(..) {
        crate::seed::debug::square::seed_debug_square(
            &queue,
            &images,
            &atlas,
            &event.slice,
            event.value,
        );
    }
}
