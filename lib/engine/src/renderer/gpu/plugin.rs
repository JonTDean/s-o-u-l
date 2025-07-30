//! engine/gpu/plugin.rs – initialises GPU compute back‑end.
//!
//! Updated 2025‑07‑28
//! * Atlas allocator now **always returns layer 0** so that
//!   newly‑spawned automata are visible immediately.
//! * First‑fit heuristic (deterministic) instead of random choice.

use bevy::{
    core_pipeline::core_2d::graph::Node2d,
    prelude::*,
    render::{
        render_graph::{RenderGraph, RenderLabel},
        render_resource::*,
        ExtractSchedule, RenderApp,
    },
};


use crate::systems::state::AppState;

use super::{
    graph::{extract_gpu_slices, ComputeAutomataNode, GpuGridSlice},
    pipelines::GpuPipelineCache,
};

const MAX_W: u32 = 1024;
const MAX_H: u32 = 1024;
const MAX_LAYERS: u32 = 256;      // still honoured for future work

/* ───────────────────── texture + atlas resources ───────────────────── */

/// Ping/pong + optional signal textures (shared by *all* automata).
#[derive(Resource)]
pub struct GlobalStateTextures {
    pub ping:   Handle<Image>,
    pub pong:   Handle<Image>,
    pub signal: Handle<Image>,
}

/* -------------------------------------------------------------------- */
/*               SIMPLE 2‑D “GUILLLOTINE” ATLAS ALLOCATOR               */
/* -------------------------------------------------------------------- */

/// Internal free‑list entry: `(offset, size)` – **layer is fixed at 0**.
#[derive(Clone, Copy)]
struct Rect {
    off:  UVec2,
    size: UVec2,
}

/// Packs 2‑D slices into the `(MAX_W × MAX_H)` atlas on **layer 0**.
/// Not thread‑safe; touched only from the main‑world.
#[derive(Resource)]
struct AtlasAllocator {
    free: Vec<Rect>,
}
impl AtlasAllocator {
    fn new() -> Self {
        Self {
            free: vec![Rect { off: UVec2::ZERO, size: UVec2::new(MAX_W, MAX_H) }],
        }
    }

    /// First‑fit guillotine split; returns `(layer == 0, offset)`.
    fn allocate(&mut self, size: UVec2) -> Option<(u32, UVec2)> {
        let idx = self
            .free
            .iter()
            .position(|r| size.x <= r.size.x && size.y <= r.size.y)?;

        let rect = self.free.remove(idx);

        /* Split right */
        if rect.size.x > size.x {
            self.free.push(Rect {
                off:  UVec2::new(rect.off.x + size.x, rect.off.y),
                size: UVec2::new(rect.size.x - size.x, size.y),
            });
        }
        /* Split below */
        if rect.size.y > size.y {
            self.free.push(Rect {
                off:  UVec2::new(rect.off.x, rect.off.y + size.y),
                size: UVec2::new(rect.size.x, rect.size.y - size.y),
            });
        }

        Some((0, rect.off))          // <- **always layer 0**
    }

    #[allow(dead_code)]
    fn free(&mut self, _layer: u32, off: UVec2, size: UVec2) {
        // Returned slices are simply appended; merging is a future task.
        self.free.push(Rect { off, size });
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
        /* 1 ── build the shared 3‑D textures (ping / pong / signal) */
        let tex_desc = |label: &'static str| Image {
            texture_descriptor: TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width:  MAX_W,
                    height: MAX_H,
                    depth_or_array_layers: MAX_LAYERS,
                },
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

        /* 2 ── attach a GPU slice whenever a new automaton appears */
        app.add_systems(
            Update,
            |mut cmd:   Commands,
             mut atlas: ResMut<AtlasAllocator>,
             mut ev:    EventReader<crate::events::AutomatonAdded>| {
                for ev in ev.read() {
                    let size = UVec2::splat(256); // TODO: real grid size in a later PR
                    if let Some((layer, offset)) = atlas.allocate(size) {
                        cmd.entity(ev.entity).insert(GpuGridSlice {
                            layer,
                            offset,
                            size,
                            rule:      "life".into(),    // TODO: pass actual rule id
                            rule_bits: 0b0001_1000,      // example B/S mask
                        });
                    } else {
                        warn!("🏳️  Atlas full – cannot allocate GPU slice!");
                    }
                }
            },
        );

        /* 3 ── wipe the allocator when returning to the main menu */
        app.add_systems(
            OnEnter(AppState::MainMenu),
            |mut atlas: ResMut<AtlasAllocator>| *atlas = AtlasAllocator::new(),
        );

        /* 4 ── render‑app side: parity‑flip + extraction + compute node */
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(FrameParity::default())
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            .add_systems(ExtractSchedule, extract_gpu_slices);

        {
            let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
            graph.add_node(AutomataStepLabel, ComputeAutomataNode);
            graph.add_node_edge(AutomataStepLabel, Node2d::MainOpaquePass);
        }
    }
}
