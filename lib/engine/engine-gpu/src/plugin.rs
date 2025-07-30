//! engine/gpu/plugin.rs ─ GPU compute back-end bootstrap for S.O.U.L.
//!
//! *Updated for Bevy 0.16.*  
//!   • `MAIN_PASS_DEPENDENCIES` / `MAIN_PASS_DRIVER` were removed.  
//!   • Sub-graphs and node labels are now *typed* (`RenderSubGraph` /
//!     `RenderLabel`).  
//!   • The Core-2D graph lives under [`bevy::core_pipeline::core_2d::graph`].  
//!   • Individual stages are enumerated in [`Node2d`] :contentReference[oaicite:0]{index=0}.
//!
//! We insert one [`ComputeAutomataNode`] between
//! `Node2d::StartMainPass` and `Node2d::EndMainPass`, guaranteeing that
//! **all** automata are stepped on the GPU *before* the main colour pass.

use bevy::{
    prelude::*,
    render::{
        render_graph::{RenderGraph, RenderLabel},
        render_resource::*,
        ExtractSchedule, RenderApp,
    },
    core_pipeline::core_2d::graph::{Core2d, Node2d},
};

use engine_core::{events::AutomatonAdded, prelude::AppState};

use crate::graph::ExtractedGpuSlices;

use super::{
    graph::{extract_gpu_slices, ComputeAutomataNode, GpuGridSlice},
    pipelines::GpuPipelineCache,
};

/* ------------------------------------------------------------------------- */
/* Compile-time configuration                                                */
/* ------------------------------------------------------------------------- */

/// Atlas dimensions – *hardware-friendly* powers of two.
const MAX_W:      u32 = 1_024;
const MAX_H:      u32 = 1_024;
const MAX_LAYERS: u32 = 256;        // reserved for future 3-D automata

/* ------------------------------------------------------------------------- */
/* Shared GPU textures (ping / pong / signal)                                */
/* ------------------------------------------------------------------------- */

/// Handles to the global 3-D textures shared by every automaton slice.
#[derive(Resource, Clone)]
pub struct GlobalStateTextures {
    pub ping:   Handle<Image>,
    pub pong:   Handle<Image>,
    pub signal: Handle<Image>,
}

/* ------------------------------------------------------------------------- */
/* Tiny 2-D guillotine atlas allocator                                       */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Copy)]
struct Rect { off: UVec2, size: UVec2 }

#[derive(Resource)]
struct AtlasAllocator { free: Vec<Rect> }

impl AtlasAllocator {
    #[inline] fn new() -> Self {
        Self { free: vec![Rect { off: UVec2::ZERO,
                                 size: UVec2::new(MAX_W, MAX_H) }] }
    }

    /// Allocate a rectangle; returns `(layer, offset)`.
    fn allocate(&mut self, size: UVec2) -> Option<(u32, UVec2)> {
        let idx  = self.free.iter().position(|r|
                       size.x <= r.size.x && size.y <= r.size.y)?;
        let rect = self.free.remove(idx);

        // Split right strip.
        if rect.size.x > size.x {
            self.free.push(Rect {
                off:  UVec2::new(rect.off.x + size.x, rect.off.y),
                size: UVec2::new(rect.size.x - size.x, size.y),
            });
        }
        // Split bottom strip.
        if rect.size.y > size.y {
            self.free.push(Rect {
                off:  UVec2::new(rect.off.x, rect.off.y + size.y),
                size: UVec2::new(rect.size.x, rect.size.y - size.y),
            });
        }
        Some((0, rect.off))            // single-layer for now
    }
}

/* ------------------------------------------------------------------------- */
/* One-bit ping/pong frame-parity resource                                   */
/* ------------------------------------------------------------------------- */

#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

/* ------------------------------------------------------------------------- */
/* Plugin definition                                                         */
/* ------------------------------------------------------------------------- */

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct AutomataStepLabel;

pub struct GpuAutomataComputePlugin;

impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ allocate the shared atlas textures --------------------------- */
        let make_image = |label: &'static str| {
            let mut img = Image {
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

            // one byte per texel ─ zero-initialise so Bevy can upload safely
            img.data = vec![0; (MAX_W * MAX_H * MAX_LAYERS) as usize].into();
            img
        };

        /* allocate the shared atlas textures --------------------------- */
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let ping   = images.add(make_image("ca.ping"));
        let pong   = images.add(make_image("ca.pong"));
        let signal = images.add(make_image("ca.signal"));
        drop(images);                       // <- release the borrow

        // Wrap the handles once
        let global_tex = GlobalStateTextures { ping: ping.clone(),
                                            pong: pong.clone(),
                                            signal: signal.clone() };

        /* ── main world ──────────────────────────────────────────────────── */
        app.insert_resource(global_tex.clone())
           .insert_resource(AtlasAllocator::new());  

        /* 2 ░ slice allocation when new automata spawn -------------------- */
        app.add_systems(
            Update,
            |mut cmd:   Commands,
             mut atlas: ResMut<AtlasAllocator>,
             mut ev:    EventReader<AutomatonAdded>| {
                for ev in ev.read() {
                    let size = UVec2::splat(256);            // TODO derive
                    if let Some((layer, offset)) = atlas.allocate(size) {
                        cmd.entity(ev.entity).insert(GpuGridSlice {
                            layer,
                            offset,
                            size,
                            rule:      "life".into(),
                            rule_bits: 0b0001_1000,
                        });
                    } else {
                        warn!("🏳️  Atlas full – cannot allocate GPU slice!");
                    }
                }
            },
        );

        /* 3 ░ allocator reset on main-menu -------------------------------- */
        app.add_systems(
            OnEnter(AppState::MainMenu),
            |mut a: ResMut<AtlasAllocator>| *a = AtlasAllocator::new(),
        );


        /* 4 ░ render-app extraction + parity flip + graph wiring ----------- */
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(global_tex);

        render_app
            .init_resource::<ExtractedGpuSlices>()
            .init_resource::<GpuPipelineCache>()
            .insert_resource(FrameParity::default())
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            .add_systems(ExtractSchedule, extract_gpu_slices);

        // ---- Graph insertion (Core-2D sub-graph) --------------------------
        {
            let mut rg =
                render_app.world_mut().resource_mut::<RenderGraph>();

            let core_2d = rg.get_sub_graph_mut(Core2d)
                .expect("Core-2D render graph missing");

            core_2d.add_node(AutomataStepLabel, ComputeAutomataNode);

            core_2d.add_node_edge(Node2d::StartMainPass, AutomataStepLabel);

            core_2d.add_node_edge(AutomataStepLabel, Node2d::EndMainPass);
        }
    }
}
