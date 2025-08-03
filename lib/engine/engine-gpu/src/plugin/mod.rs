// lib/engine/engine-gpu/src/plugin/mod.rs
//! GPU back-end **bootstrap plugin** for Automatoxel.
//!
//! * Promotes the mesh-shader path at run-time when both the Cargo feature
//!   _and_ the Vulkan driver capability are present.
//! * Tracks per-frame simulation statistics so the UI layer can display
//!   step-counts and rough GPU time.

#![allow(clippy::type_complexity)]

#[cfg(feature = "gpu-debug")]
use bevy::render::texture::GpuImage;
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraph, RenderGraphContext},
        render_resource::{
            CachedRenderPipelineId, FragmentState, MultisampleState, PipelineCache,
            PrimitiveState, RenderPipelineDescriptor, TextureFormat,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode,
        },
        renderer::RenderDevice,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};
use wgpu::{ColorTargetState, ColorWrites, Features};

use engine_core::{
    automata::GpuGridSlice,
    events::{AutomatonAdded, DebugSeedSquare},
    prelude::{AppState, AutomataRegistry},
    systems::simulation::SimulationStep,
};

#[cfg(feature = "gpu-debug")]
use crate::seed::debug::square::seed_debug_square;

pub mod atlas;
pub mod labels;
pub mod textures;

use atlas::AtlasAllocator;
use labels::*;
use textures::{make_atlas, make_image};

use crate::{
    compute::dual_contour::{DualContourNode, MeshletBuffers, MAX_VOXELS},
    graph::{extract_gpu_slices, ComputeAutomataNode, ExtractedGpuSlices},
    pipelines::GpuPipelineCache,
};

#[cfg(feature = "mesh_shaders")]
use crate::compute::mesh_path::MeshPathNode;

/* ───────────────── Resources ───────────────── */
#[derive(Resource, Clone)]
pub struct GlobalVoxelAtlas {
    pub atlas: Handle<Image>,
    pub signal: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

#[derive(Resource, Default, Clone, Copy)]
pub struct StepsThisFrame {
    pub steps: u32,
    pub gpu_time_ms: f32,
}

/* ───────────────── Helpers ───────────────── */
fn count_sim_steps(mut cmd: Commands, mut ev: ResMut<Events<SimulationStep>>) {
    let steps = ev.drain().count() as u32;
    cmd.insert_resource(StepsThisFrame {
        steps,
        gpu_time_ms: 0.0,
    });
}

fn init_meshlet_buffers(mut cmd: Commands, dev: Res<RenderDevice>) {
    cmd.insert_resource(MeshletBuffers::new(&*dev, MAX_VOXELS));
}

/* Tiny voxel debug pipeline ------------------------------------------------ */
#[derive(Resource)]
pub struct VoxelPipeline {
    pub id: CachedRenderPipelineId,
}

fn init_voxel_pipeline(
    mut cmd: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    cache: ResMut<PipelineCache>,
) {
    let shader = shaders.add(Shader::from_wgsl(
        include_str!("../../../../../assets/shaders/automatoxel/voxel_mesh.wgsl"),
        "voxel_mesh",
    ));

    let desc = RenderPipelineDescriptor {
        label: Some("voxel_mesh".into()),
        layout: vec![],
        vertex: VertexState {
            shader: shader.clone(),
            entry_point: "vertex".into(),
            shader_defs: vec![],
            buffers: vec![VertexBufferLayout {
                array_stride: 24, // pos + nrm
                step_mode: VertexStepMode::Vertex,
                attributes: vec![
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                ],
            }],
        },
        fragment: Some(FragmentState {
            shader,
            entry_point: "fragment".into(),
            shader_defs: vec![],
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Bgra8UnormSrgb,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
        zero_initialize_workgroup_memory: false,
    };

    cmd.insert_resource(VoxelPipeline {
        id: cache.queue_render_pipeline(desc),
    });
}

/* Compile WGSL compute pipelines at start-up ------------------------ */
fn init_compute_pipelines(
    dev: Res<RenderDevice>,
    mut gp: ResMut<GpuPipelineCache>,
    mut sh: ResMut<Assets<Shader>>,
    mut pc: ResMut<PipelineCache>,
) {
    let dc_src = include_str!("../../../../../assets/shaders/automatoxel/dual_contour.wgsl");
    gp.get_or_create("dual_contour", dc_src, &mut pc, &mut sh, &*dev);

    #[cfg(feature = "mesh_shaders")]
    {
        let mp_src = include_str!("../../../../../assets/shaders/automatoxel/mesh_path.wgsl");
        gp.get_or_create("mesh_path", mp_src, &mut pc, &mut sh, &*dev);
    }
}

/* Optional GPU debug helper ----------------------------------------- */
#[cfg(feature = "gpu-debug")]
fn render_debug_squares(
    mut ev: EventReader<DebugSeedSquare>,
    queue: Res<bevy::render::renderer::RenderQueue>,
    images: Res<bevy::render::render_asset::RenderAssets<GpuImage>>,
    atlas: Res<GlobalVoxelAtlas>,
) {
    for dbg in ev.read() {
        seed_debug_square(&queue, &images, &atlas, &dbg.slice, dbg.value);
    }
}

/* Dummy node to keep legacy frame inspectors happy ------------------ */
pub struct VoxelDrawNode;
impl bevy::render::render_graph::Node for VoxelDrawNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        _ctx: &mut bevy::render::renderer::RenderContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        Ok(())
    }
}

/* ───────────────── Plugin body ───────────────── */
pub struct GpuAutomataComputePlugin;

impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        /* 0 ░ Main-world events */
        app.add_event::<DebugSeedSquare>()
            .add_event::<SimulationStep>();

        /* 1 ░ Global atlas + allocator */
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let atlas = images.add(make_atlas("ca.atlas"));
        let signal = images.add(make_image("ca.signal"));

        app.insert_resource(GlobalVoxelAtlas {
            atlas: atlas.clone(),
            signal: signal.clone(),
        })
        .insert_resource(AtlasAllocator::default());

        /* 2 ░ Slice allocator per automaton */
        app.add_systems(
            Update,
            |mut cmd: Commands,
             mut alloc: ResMut<AtlasAllocator>,
             mut ev: EventReader<AutomatonAdded>,
             mut reg: ResMut<AutomataRegistry>,
             slice_q: Query<&GpuGridSlice>| {
                for ev in ev.read() {
                    if slice_q.get(ev.entity).is_ok() {
                        continue;
                    }
                    let Some(info) = reg.get_mut(ev.id) else { continue };
                    let want = info.slice.size.max(UVec2::splat(256));
                    if let Some((layer, off)) = alloc.allocate(want) {
                        let slice = GpuGridSlice {
                            layer,
                            offset: off,
                            size: want,
                            rule: info.name.clone(),
                            rule_bits: 0,
                        };
                        cmd.entity(ev.entity).insert(slice.clone());
                        info.slice = slice.clone();
                        info.world_offset =
                            Vec3::new(off.x as f32, off.y as f32, 0.0) * info.voxel_size;
                    } else {
                        warn!("atlas full – cannot allocate {}", info.name);
                    }
                }
            },
        );

        /* 3 ░ Reset allocator on main-menu */
        app.add_systems(
            OnEnter(AppState::MainMenu),
            |mut a: ResMut<AtlasAllocator>| *a = AtlasAllocator::default(),
        );

        /* 4 ░ Render sub-app -------------------------------------------------- */
        let render_app = app.sub_app_mut(RenderApp);

        // Compute mesh-shader capability *before* mutating the render world
        const MESH_CANDIDATE_FEATURE: Features = Features::VERTEX_WRITABLE_STORAGE;
        let mesh_ok = render_app
            .world()
            .get_resource::<RenderDevice>()
            .map_or(false, |d| d.features().contains(MESH_CANDIDATE_FEATURE));

        render_app
            .insert_resource(GlobalVoxelAtlas { atlas, signal })
            .add_event::<DebugSeedSquare>()
            .add_event::<SimulationStep>()
            .init_resource::<ExtractedGpuSlices>()
            .init_resource::<GpuPipelineCache>()
            .init_resource::<StepsThisFrame>()
            .insert_resource(FrameParity::default())
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            .add_systems(ExtractSchedule, (extract_gpu_slices, count_sim_steps))
            .add_systems(
                ExtractSchedule,
                (
                    |mut _cmd: Commands, mut ev: EventReader<DebugSeedSquare>| ev.clear(),
                    |mut _cmd: Commands, mut ev: EventReader<SimulationStep>| ev.clear(),
                ),
            )
            .add_systems(
                Startup,
                (init_meshlet_buffers, init_voxel_pipeline, init_compute_pipelines),
            )
            .configure_sets(Render, (RenderSet::Queue,));

        #[cfg(feature = "gpu-debug")]
        render_app.add_systems(Render, render_debug_squares.after(RenderSet::Queue));

        /* 5 ░ Render-graph wiring */
        let mut graph = render_app
            .world_mut()
            .resource_mut::<RenderGraph>();
        let core3d_sub = graph.get_sub_graph_mut(Core3d).unwrap();
        core3d_sub.add_node(AutomataStepLabel, ComputeAutomataNode);

        if mesh_ok {
            #[cfg(feature = "mesh_shaders")]
            {
                core3d_sub.add_node(MeshPathLabel, MeshPathNode);
                core3d_sub.add_node_edge(Node3d::StartMainPass, AutomataStepLabel);
                core3d_sub.add_node_edge(AutomataStepLabel, MeshPathLabel);
                core3d_sub.add_node_edge(MeshPathLabel, Node3d::EndMainPass);
            }
        } else {
            core3d_sub.add_node(DualContourLabel, DualContourNode);
            core3d_sub.add_node(DrawVoxelLabel, VoxelDrawNode);
            core3d_sub.add_node_edge(Node3d::StartMainPass, AutomataStepLabel);
            core3d_sub.add_node_edge(AutomataStepLabel, DualContourLabel);
            core3d_sub.add_node_edge(DualContourLabel, DrawVoxelLabel);
            core3d_sub.add_node_edge(DrawVoxelLabel, Node3d::EndMainPass);
        }
    }
}
