//! engine/gpu/plugin.rs – initialises GPU compute back‑end.

use bevy::{
    prelude::*,
    core_pipeline::core_2d::graph::Node2d,
    render::{
        render_resource::*,
        ExtractSchedule, RenderApp,
        render_graph::{RenderGraph, RenderLabel},
    },
};

use crate::state::AppState;

use super::{
    graph::{ComputeAutomataNode, extract_gpu_slices, GpuGridSlice},
    pipelines::GpuPipelineCache,
};

const MAX_W:      u32 = 1024;
const MAX_H:      u32 = 1024;
const MAX_LAYERS: u32 = 256;

/* ───────────────────── texture + atlas resources ───────────────────── */

/// Ping/pong + optional signal textures (shared by *all* automata).
#[derive(Resource)]
pub struct GlobalStateTextures {
    pub ping:   Handle<Image>,
    pub pong:   Handle<Image>,
    pub signal: Handle<Image>,
}

/// Simple guillotine allocator for packing 2‑D slices into the texture
/// atlas.  **Not thread‑safe**; accessed from the main world only.
#[derive(Resource)]
struct AtlasAllocator {
    free_spaces: Vec<(u32, UVec2, UVec2)>, // (layer, offset, size)
}
impl AtlasAllocator {
    fn new() -> Self {
        Self {
            free_spaces: vec![(0, UVec2::ZERO, UVec2::new(MAX_W, MAX_H))],
        }
    }
    fn allocate(&mut self, size: UVec2) -> Option<(u32, UVec2)> {
        let idx = self
            .free_spaces
            .iter()
            .position(|&(_, _, free)| size.x <= free.x && size.y <= free.y)?;
        let (layer, off, free) = self.free_spaces.remove(idx);

        // split right
        if free.x > size.x {
            self.free_spaces.push((
                layer,
                UVec2::new(off.x + size.x, off.y),
                UVec2::new(free.x - size.x, size.y),
            ));
        }
        // split below
        if free.y > size.y {
            self.free_spaces.push((
                layer,
                UVec2::new(off.x, off.y + size.y),
                UVec2::new(free.x, free.y - size.y),
            ));
        }
        Some((layer, off))
    }
    fn free(&mut self, layer: u32, off: UVec2, size: UVec2) {
        self.free_spaces.push((layer, off, size));
        // TODO merge free rectangles to fight fragmentation
    }
}

/// One‑bit flag: `false` → read = ping, `true` → read = pong.
#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

/* ─────────────────────────── plugin impl ───────────────────────────── */

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct AutomataStepLabel;

pub struct GpuAutomataComputePlugin;
impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        /* 1 create the 3‑D textures */
        let tex_desc = |label: &'static str| Image {
            texture_descriptor: TextureDescriptor {
                label: Some(label),
                size:  Extent3d { width: MAX_W, height: MAX_H, depth_or_array_layers: MAX_LAYERS },
                mip_level_count: 1,
                sample_count:    1,
                dimension:       TextureDimension::D3,
                format:          TextureFormat::R8Uint,
                usage: TextureUsages::STORAGE_BINDING
                     | TextureUsages::COPY_DST
                     | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            ..default()
        };
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let ping   = images.add(tex_desc("ca.ping"));
        let pong   = images.add(tex_desc("ca.pong"));
        let signal = images.add(tex_desc("ca.signal"));
        drop(images);

        app.insert_resource(GlobalStateTextures { ping, pong, signal })
           .insert_resource(AtlasAllocator::new())
           .insert_resource(GpuPipelineCache::default());

        /* 2 attach a GPU slice when a new automaton entity appears */
        app.add_systems(
            Update,
            |mut cmd: Commands,
             mut atlas: ResMut<AtlasAllocator>,
             mut ev:   EventReader<crate::events::AutomatonAdded>| {
                for ev in ev.read() {
                    let size = UVec2::splat(256); // TODO real size
                    if let Some((layer, offset)) = atlas.allocate(size) {
                        cmd.entity(ev.entity).insert(GpuGridSlice {
                            layer,
                            offset,
                            size,
                            rule: "life".into(),    // TODO from automaton meta
                            rule_bits: 0b0001_1000, // B/S mask – example
                        });
                    } else {
                        warn!("🏳️  Atlas full – cannot allocate GPU slice!");
                    }
                }
            },
        );

        /* 3 wipe the slice allocator when returning to main‑menu */
        app.add_systems(
            OnEnter(AppState::MainMenu),
            |mut atlas: ResMut<AtlasAllocator>| *atlas = AtlasAllocator::new(),
        );

        /* 4  move data into the render‑app & add compute node */
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(FrameParity::default())
            // flip parity every extract
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            // transfer component
            .add_systems(ExtractSchedule, extract_gpu_slices);

        {
            let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
            graph.add_node(AutomataStepLabel, ComputeAutomataNode);
            graph.add_node_edge(AutomataStepLabel, Node2d::MainOpaquePass);
        }


    }
}
