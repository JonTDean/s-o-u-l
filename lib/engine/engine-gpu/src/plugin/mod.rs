// lib/engine/engine-gpu/src/plugin/mod.rs
//! GPU back-end **bootstrap plugin** for *Automatoxel*.
//!
//! This module wires the simulation → GPU → raster pipeline together and
//! exposes one public plug-in, [`GpuAutomataComputePlugin`].  Attach it to
//! your *main* `App` **before** any rule plug-ins so that shader pipelines
//! and atlas resources are ready when the first automaton is spawned.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use engine_gpu::GpuAutomataComputePlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(GpuAutomataComputePlugin)   // ← here
//!     .run();
//! ```
//!
//! ### What changed in this file?
//!
//! * `EventReader::drain()` does **not** exist in Bevy 0.16.x.  
//!   We now count events with `ev.read().count()` (which automatically marks
//!   them as read for this reader).  No manual draining is needed.
//!
//! Everything else remains fully multi-threaded, order-agnostic and matches
//! the version you posted earlier.

#![allow(clippy::type_complexity)]

use std::collections::HashSet;

use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        render_graph::RenderGraph, render_resource::{
            CachedRenderPipelineId, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, TextureFormat, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        }, renderer::RenderDevice, texture::GpuImage, ExtractSchedule, Render, RenderApp, RenderSet
    },
};
use wgpu::{ColorTargetState, ColorWrites};

use engine_core::{
    automata::GpuGridSlice,
    events::{AutomatonAdded, DebugSeedSquare},
    prelude::{AppState, AutomataRegistry},
    systems::simulation::SimulationStep,
};

#[cfg(feature = "gpu-debug")]
use bevy::render::render_asset::RenderAssets;

#[cfg(feature = "gpu-debug")]
use crate::seed::debug::square::seed_debug_square;

/* internal sub-modules */
pub mod atlas;
/// Internal Node Labeling for gpu logic
pub mod labels;
pub mod textures;
pub mod voxel_draw_node;

use atlas::AtlasAllocator;
use labels::*;
use textures::{make_atlas, make_image};

use crate::{
    compute::dual_contour::{DualContourNode, MeshletBuffers, MAX_VOXELS},
    graph::{extract_gpu_slices, ComputeAutomataNode, ExtractedGpuSlices},
    pipelines::GpuPipelineCache,
    plugin::voxel_draw_node::VoxelDrawNode, seed::lenia::seed_orbium,
};

#[cfg(feature = "mesh_shaders")]
use crate::compute::mesh_path::MeshPathNode;

/* ──────────────────────────────────────────────────────────────── *
 *                        Public resources                         *
 * ──────────────────────────────────────────────────────────────── */

/// Handles for the global voxel-atlas textures.
#[derive(Resource, Clone)]
pub struct GlobalVoxelAtlas {
    /// 3-D texture storing automaton state (R8Uint).
    pub atlas:  Handle<Image>,
    /// 2-D signalling texture (same X/Y footprint as one atlas layer).
    pub signal: Handle<Image>,
}

/// Toggles even/odd frames for ping-pong resources.
#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

/// Per-frame simulation statistics copied into the render world.
#[derive(Resource, Default, Clone, Copy)]
pub struct StepsThisFrame {
    /// Number of simulation ticks executed *this* frame.
    pub steps:       u32,
    /// GPU time spent stepping automatons (ms) – filled by the graph.
    pub gpu_time_ms: f32,
}

#[derive(Resource, Default)]
struct SeededSlices(HashSet<(u32, u32, u32)>);         // (layer, off.x, off.y)

/* ──────────────────────────────────────────────────────────────── *
 *                    Helper initialisation code                   *
 * ──────────────────────────────────────────────────────────────── */

/// Counts all `SimulationStep` events that were **generated this frame**
/// in the **main** world.
/// 
/// `EventReader::read()` yields only the *new* events since the last call  
/// and automatically marks them as read for this reader, so no explicit
/// draining is required.
fn count_sim_steps_main(
    mut ev:    EventReader<SimulationStep>,
    mut stats: ResMut<StepsThisFrame>,
) {
    stats.steps = ev.read().count() as u32;
}

/// Mirrors [`StepsThisFrame`] into the render world so render-graph nodes
/// can access it without cross-world event plumbing.
fn extract_steps_this_frame(steps: Res<StepsThisFrame>, mut cmd: Commands) {
    cmd.insert_resource(*steps);
}

/// Allocates the scratch buffers used by the dual-contouring compute pass.
fn init_meshlet_buffers(mut cmd: Commands, dev: Res<RenderDevice>) {
    cmd.insert_resource(MeshletBuffers::new(&*dev, MAX_VOXELS));
}

/// Compiles all *fixed* WGSL compute pipelines (dual-contour, mesh-path, …)
/// and caches them in [`GpuPipelineCache`].
fn init_compute_pipelines(
    dev:      Res<RenderDevice>,
    mut gp:   ResMut<GpuPipelineCache>,
    mut sh:   ResMut<Assets<Shader>>,
    mut pc:   ResMut<PipelineCache>,
) {
    let dc_src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
       "/assets/dual_contour.wgsl"
    ));

    gp.get_or_create("dual_contour", dc_src, &mut pc, &mut sh, &*dev);

    {
        let mp_src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/mesh/mesh_path.wgsl"
        ));

        gp.get_or_create("mesh_path", mp_src, &mut pc, &mut sh, &*dev);
    }

    let lenia_src =
        include_str!("../../../../../assets/shaders/gpu/automata/lenia_step.wgsl");
    gp.get_or_create("lenia", lenia_src, &mut pc, &mut sh, &*dev);
    gp.get_or_create("lenia:orbium", lenia_src, &mut pc, &mut sh, &*dev);
}

/* ──────────────────────────────────────────────────────────────── *
 *                Wire-frame voxel debug draw pipeline             *
 * ──────────────────────────────────────────────────────────────── */

/// Pipeline that renders the debug voxel-meshlet stream.
#[derive(Resource)]
pub struct VoxelPipeline {
    /// Cached Bevy render-pipeline identifier.
    pub id: CachedRenderPipelineId,
}

fn init_voxel_pipeline(
    mut cmd:     Commands,
    mut shaders: ResMut<Assets<Shader>>,
    cache:   ResMut<PipelineCache>,
) {
    let shader = shaders.add(Shader::from_wgsl(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/voxel_mesh.wgsl"
        )),
        "voxel_mesh",
    ));

    // 32-byte vertex stride (vec4 pos + vec4 nrm)
    let desc = RenderPipelineDescriptor {
        label: Some("voxel_mesh".into()),
        layout: vec![],                    // automatic: view bind-group is group 0
        vertex: VertexState {
            shader:        shader.clone(),
            entry_point:   "vertex".into(),
            shader_defs:   vec![],
            buffers: vec![VertexBufferLayout {
                array_stride: 32,          // 2 × vec4<f32>
                step_mode:    VertexStepMode::Vertex,
                attributes: vec![
                    VertexAttribute { format: VertexFormat::Float32x4, offset: 0,  shader_location: 0 },
                    VertexAttribute { format: VertexFormat::Float32x4, offset: 16, shader_location: 1 },
                ],
            }],
        },
        fragment: Some(FragmentState {
            shader,
            entry_point: "fragment".into(),
            shader_defs: vec![],
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Bgra8UnormSrgb,
                blend:  None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive:     PrimitiveState::default(),
        depth_stencil: None,
        multisample:   MultisampleState::default(),
        push_constant_ranges: vec![],
        zero_initialize_workgroup_memory: false,
    };

    cmd.insert_resource(VoxelPipeline { id: cache.queue_render_pipeline(desc) });
}

/* ──────────────────────────────────────────────────────────────── *
 *                 Optional GPU-debug square helper                *
 * ──────────────────────────────────────────────────────────────── */

#[cfg(feature = "gpu-debug")]
fn render_debug_squares(
    mut ev:    EventReader<DebugSeedSquare>,
    queue:     Res<bevy::render::renderer::RenderQueue>,
    images:    Res<RenderAssets<GpuImage>>,
    atlas:     Res<GlobalVoxelAtlas>,
) {
    for dbg in ev.read() {
        seed_debug_square(&queue, &images, &atlas, &dbg.slice, dbg.value);
    }
}

/* ──────────────────────────────────────────────────────────────── *
 *                         Main plug-in                            *
 * ──────────────────────────────────────────────────────────────── */

/// Boots the GPU compute back-end and integrates all render-graph nodes.
pub struct GpuAutomataComputePlugin;

impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        /* 0 ░ Global events & resources in the main world  ─────────── */

        app.add_event::<DebugSeedSquare>()
            .add_event::<SimulationStep>()
            .init_resource::<StepsThisFrame>()          // shared counter
            .add_systems(Update, count_sim_steps_main); // tick counter

        /* 1 ░ Global voxel atlas  ──────────────────────────────────── */

        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let atlas  = images.add(make_atlas("ca.atlas"));
        let signal = images.add(make_image("ca.signal"));

        app.insert_resource(GlobalVoxelAtlas { atlas: atlas.clone(), signal: signal.clone() })
            .insert_resource(AtlasAllocator::default());

        /* 2 ░ Slice allocator reacts to AutomatonAdded events ─────── */

        app.add_systems(
            Update,
            |mut cmd:  Commands,
             mut alloc: ResMut<AtlasAllocator>,
             mut ev:   EventReader<AutomatonAdded>,
             mut reg:  ResMut<AutomataRegistry>,
             slices:   Query<&GpuGridSlice>| {
                for ev in ev.read() {
                    if slices.get(ev.entity).is_ok() { continue; }       // already has slice
                    let Some(info) = reg.get_mut(ev.id) else { continue };
                    let want = info.slice.size.max(UVec2::splat(256));
                    if let Some((layer, off)) = alloc.allocate(want) {
                        /* --------- FIX: no more `dimension`; preserve stored depth -------- */
                        let slice = GpuGridSlice {
                            layer,
                            offset: off,
                            size:   want,
                            depth:  info.slice.depth.max(1),   // ← was `info.dimension…`
                            rule:   info.name.clone(),
                            rule_bits: 0,
                        };

                        cmd.entity(ev.entity).insert(slice.clone());
                        info.slice        = slice.clone();
                        info.world_offset =
                            Vec3::new(off.x as f32, off.y as f32, 0.0) * info.voxel_size;
                    } else {
                        warn!("voxel atlas is full – {}", info.name);
                    }
                }
            },
        );

        /* 3 ░ Reset allocator on MainMenu ──────────────────────────── */

        app.add_systems(OnEnter(AppState::MainMenu), |mut a: ResMut<AtlasAllocator>| {
            *a = AtlasAllocator::default()
        });

        /* 4 ░ Render sub-app bootstrap ─────────────────────────────── */

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            // shared atlas handles
            .insert_resource(GlobalVoxelAtlas { atlas, signal })
            // mirrored events so UI can send debug squares from main world
            .add_event::<DebugSeedSquare>()
            .add_event::<SimulationStep>()
            // per-frame resources
            .init_resource::<ExtractedGpuSlices>()
            .init_resource::<GpuPipelineCache>()
            .init_resource::<StepsThisFrame>()
            .insert_resource(FrameParity::default())

            // extractor schedule
            .add_systems(ExtractSchedule, (extract_gpu_slices, extract_steps_this_frame))
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            .add_systems(ExtractSchedule,
                (|mut ev: EventReader<DebugSeedSquare>| ev.read().for_each(drop),
                 |mut ev: EventReader<SimulationStep>|  ev.read().for_each(drop))
            )
            // startup
            .add_systems(Startup,
                (init_meshlet_buffers,
                 init_voxel_pipeline,
                 init_compute_pipelines))
            .configure_sets(Render, (RenderSet::Queue,))
            .init_resource::<SeededSlices>()                 
            .add_systems(                                    
                Render,
                seed_new_lenia_slices.in_set(RenderSet::Queue),
            );

        #[cfg(feature = "gpu-debug")]
        render_app.add_systems(Render, render_debug_squares.after(RenderSet::Queue));

        /* 5 ░ Render-graph wiring (Core3D sub-graph) ───────────────── */

        let mut graph      = render_app.world_mut().resource_mut::<RenderGraph>();
        let core3d_subgraph = graph.get_sub_graph_mut(Core3d).expect("Core3D graph");

        // Compute step – always present
        core3d_subgraph.add_node(AutomataStepLabel, ComputeAutomataNode);
        core3d_subgraph.add_node_edge(Node3d::StartMainPass, AutomataStepLabel);

        // ── vertex generation ───────────────────────────────
        core3d_subgraph.add_node(DualContourLabel, DualContourNode);
        core3d_subgraph.add_node_edge(AutomataStepLabel, DualContourLabel);

        // ── post-process + draw ─────────────────────────────
        core3d_subgraph.add_node(MeshPathLabel,  MeshPathNode);
        core3d_subgraph.add_node_edge(DualContourLabel, MeshPathLabel);

        core3d_subgraph.add_node(DrawVoxelLabel,  VoxelDrawNode);
        core3d_subgraph.add_node_edge(MeshPathLabel,  DrawVoxelLabel);
        core3d_subgraph.add_node_edge(DrawVoxelLabel, Node3d::EndMainPass);

        // ───────────────────────────────────────────────────────────────
        // 6 ░ One-shot seeding for Lenia-family automata
        // ───────────────────────────────────────────────────────────────
        /// Seeds every brand-new Lenia slice exactly once.
        fn seed_new_lenia_slices(
            slices:   Res<ExtractedGpuSlices>,
            mut done: ResMut<SeededSlices>,
            queue:    Res<bevy::render::renderer::RenderQueue>,
            images:   Res<bevy::render::render_asset::RenderAssets<GpuImage>>,
            atlas:    Res<GlobalVoxelAtlas>,
        ) {
            for s in &slices.0 {
                let key = (s.0.layer, s.0.offset.x, s.0.offset.y);
                if done.0.contains(&key) { continue; }

                match s.0.rule.as_str() {
                    "lenia" | "lenia:orbium" => {
                        seed_orbium(&queue, &images, &atlas, s);
                    }
                    _ => {}
                }
                done.0.insert(key);
            }
        }

    }
}
