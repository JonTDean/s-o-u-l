//! Per-channel voxel ramps: up to eight world rectangles, matched in the fragment shader.
use bevy::{prelude::*, render::Extract};
use bytemuck::{Pod, Zeroable};
use engine_core::automata::{
    GpuGridSlice,
    channel::{ChannelId, ChannelSettings, Channels, Srgb, ramp},
};

/// Maximum distinct channel ramps per frame (the channel capacity).
pub(crate) const MAX_ENTRIES: usize = 8;

/// One instance's world-space XY rectangle and four linear-RGB ramp stops.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub(crate) struct PaletteEntry {
    pub rect: [f32; 4],
    pub stops: [[f32; 4]; 4],
}

/// Uniform block; `header.x` is the entry count, the rest is padding.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct PaletteUniform {
    pub header: [u32; 4],
    pub entries: [PaletteEntry; MAX_ENTRIES],
}

/// Render-world palette for the current frame.
#[derive(Resource, Clone, Copy)]
pub(crate) struct VoxelPalette(pub PaletteUniform);

impl Default for VoxelPalette {
    fn default() -> Self {
        Self(build([]))
    }
}

/// Pack up to eight `(rect, colour)` pairs; earlier entries win where rectangles overlap.
pub(crate) fn build(entries: impl IntoIterator<Item = ([f32; 4], Srgb)>) -> PaletteUniform {
    let mut uniform = PaletteUniform {
        header: [0; 4],
        entries: [PaletteEntry::default(); MAX_ENTRIES],
    };
    for (slot, (rect, colour)) in uniform.entries.iter_mut().zip(entries) {
        slot.rect = rect;
        slot.stops = ramp(colour).map(|stop| {
            let [r, g, b] = stop.linear();
            [r, g, b, 1.0]
        });
        uniform.header[0] += 1;
    }
    uniform
}

/// World XY rectangle covered by a slice, matching `VoxelSurfaceNode`'s 4-unit voxels.
pub(crate) fn rect(slice: &GpuGridSlice) -> [f32; 4] {
    let (x, y) = (slice.offset.x as f32 * 4.0, slice.offset.y as f32 * 4.0);
    [
        x,
        y,
        x + slice.size.x as f32 * 4.0,
        y + slice.size.y as f32 * 4.0,
    ]
}

/// Copy channel colours into the render world, selected channel first.
pub(crate) fn extract(
    owners: Extract<Query<(&ChannelId, &ChannelSettings, &GpuGridSlice)>>,
    channels: Extract<Option<Res<Channels>>>,
    mut palette: ResMut<VoxelPalette>,
) {
    let selected = channels.as_ref().and_then(|c| c.selected);
    let mut entries: Vec<_> = owners.iter().collect();
    entries.sort_by_key(|(id, ..)| (Some(**id) != selected, **id));
    let pairs = entries
        .into_iter()
        .map(|(_, settings, slice)| (rect(slice), settings.colour));
    palette.0 = build(pairs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::automata::channel::{SWATCHES, Srgb, ramp};

    #[test]
    fn layout_is_656_bytes_with_eighty_byte_entries() {
        assert_eq!(std::mem::size_of::<PaletteEntry>(), 80);
        assert_eq!(std::mem::size_of::<PaletteUniform>(), 656);
    }

    #[test]
    fn build_stores_linear_ramp_stops_and_caps_at_eight() {
        let entries = (0..10).map(|i| ([i as f32, 0.0, i as f32 + 1.0, 1.0], SWATCHES[i % 5]));
        let uniform = build(entries);
        assert_eq!(uniform.header[0], 8);
        let expected = ramp(SWATCHES[0]).map(Srgb::linear);
        for (stop, want) in uniform.entries[0].stops.iter().zip(expected) {
            assert_eq!(&stop[..3], &want[..]);
            assert_eq!(stop[3], 1.0);
        }
        assert_eq!(
            build([]).header[0],
            0,
            "empty palette falls back to shader default"
        );
    }

    #[test]
    fn rect_matches_surface_world_mapping() {
        let slice = GpuGridSlice {
            layer: 2,
            offset: UVec2::new(64, 0),
            size: UVec2::splat(64),
            depth: 8,
            rule: "lenia".into(),
            rule_bits: 0,
        };
        assert_eq!(rect(&slice), [256.0, 0.0, 512.0, 256.0]);
    }

    #[test]
    fn shader_default_ramp_matches_signal() {
        let shader = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/voxel_mesh.wgsl"
        ));
        for stop in ramp(engine_core::automata::channel::SIGNAL).map(Srgb::linear) {
            let literal = format!("vec3<f32>({:.4}, {:.4}, {:.4})", stop[0], stop[1], stop[2]);
            assert!(shader.contains(&literal), "missing {literal}");
        }
    }
}
