//! engine-gpu ▸ **plugin**
//!
//! Boot-straps the GPU back-end for voxel automata:
//! * creates the global ping/pong/signalling textures,
//! * manages the 3-D atlas allocator,
//! * wires the compute & draw nodes into `Core3d`,
//! * owns the transient meshlet buffers used by Dual Contour.
//!
//! The file is large but follows a strict **top-to-bottom** structure:
//! globals → helpers → startup systems → main `Plugin` impl.

use bevy::{
    core_pipeline::core_3d::graph::Core3d, prelude::*, 
    render::{
        render_graph::{RenderGraph, RenderGraphContext}, 
        render_resource::*, 
        /* ── bring the helper into scope ─────────────────────────── */
        renderer::RenderDevice, 
        ExtractSchedule, Render, RenderApp, RenderSet,
     }
};

use engine_core::{
    automata::GpuGridSlice, events::{AutomatonAdded, DebugSeedSquare}, prelude::{AppState, AutomataRegistry}, systems::simulation::SimulationStep          // ← canonical slice type
};

#[cfg(feature = "gpu-debug")]
use crate::seed::debug::square::seed_debug_square;

pub mod atlas;
pub mod labels;
pub mod textures;

use atlas::AtlasAllocator;
#[cfg(feature = "mesh_shaders")]
use labels::MeshPathLabel;


use crate::{
    compute::dual_contour::{MeshletBuffers, MAX_VOXELS},
    graph::{extract_gpu_slices, ComputeAutomataNode, ExtractedGpuSlices},
    pipelines::GpuPipelineCache, plugin::{labels::AutomataStepLabel, textures::{make_atlas, make_image}},
};

/* =================================================================== */
/* Generic “main → render world” event forwarder                       */
/* =================================================================== */
/// Very small stand-in for Bevy’s `extract_events`.
///
/// It *moves* every event of type `T` that was emitted in the main-world
/// frame into a **fresh** `Events<T>` resource in the render-world.  
/// (Exactly what the original helper does.)
fn extract_events<T: Event + Clone>(
    mut commands: bevy::prelude::Commands,
    mut src:      bevy::prelude::ResMut<bevy::prelude::Events<T>>,
) {
    // Drain all events that were just written by the main world.
    let drained: Vec<T> = src.drain().collect();

    // Nothing to forward?
    if drained.is_empty() {
        return;
    }

    // Create a brand-new Events<T> resource for the render-world and
    // re-send everything so typical `EventReader` logic works there.
    let mut dst = bevy::prelude::Events::<T>::default();
    for e in drained {
        dst.send(e);
    }
    commands.insert_resource(dst);
}

/* ------------------------------------------------------------------- */
/* Globals & misc-helpers                                              */
/* ------------------------------------------------------------------- */

/// Handles to the sparse 3-D atlas  signalling texture.
#[derive(Resource, Clone)]
pub struct GlobalVoxelAtlas {
    pub atlas : Handle<Image>,
    pub signal: Handle<Image>,
}

/// Flips every frame → parity-based buffer indexing on the GPU.
#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

/* ── how many fixed-steps happened this frame? ───────────────── */
#[derive(Resource, Default, Clone, Copy)]
pub struct StepsThisFrame(pub u32);

fn count_sim_steps(
    mut cmd: Commands,
    mut ev:  ResMut<Events<SimulationStep>>,
) {
    let n = ev.drain().count() as u32;
    cmd.insert_resource(StepsThisFrame(n));
}

/* tiny helper that initialises the compact voxel mesh pipeline */
fn init_voxel_pipeline(
    mut cmds:     Commands,
    mut shaders:  ResMut<Assets<Shader>>,
    mut pipelines: ResMut<PipelineCache>,
) {
    let vp = VoxelPipeline::new(&mut *shaders, &mut *pipelines);
    cmds.insert_resource(vp);
}

/* ------------------------------------------------------------------- */
/* Compact voxel mesh pipeline (draw pass)                             */
/* ------------------------------------------------------------------- */

#[derive(Resource)]
pub struct VoxelPipeline {
    pub id: CachedRenderPipelineId,
}

impl VoxelPipeline {
    fn new(
        shaders:   &mut Assets<Shader>,
        pipelines: &mut PipelineCache,
    ) -> Self {
        /* 1 ─ WGSL shader */
        let shader = shaders.add(Shader::from_wgsl(
            include_str!("../../../../../assets/shaders/automatoxel/voxel_mesh.wgsl"),
            "voxel_mesh",
        ));

        /* 2 ─ Pipeline descriptor */
        let desc = RenderPipelineDescriptor {
            label: Some("voxel_mesh".into()),
            layout: vec![],
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![VertexBufferLayout {
                    array_stride: 3 * 4 + 3 * 4 + 4, // pos + nrm + mat
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![
                        VertexAttribute { format: VertexFormat::Float32x3, offset: 0,  shader_location: 0 },
                        VertexAttribute { format: VertexFormat::Float32x3, offset: 12, shader_location: 1 },
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

        let id = pipelines.queue_render_pipeline(desc);
        Self { id }
    }
}

/* ------------------------------------------------------------------- */
/* Voxel draw node – consumes Dual-Contour meshlets                    */
/* ------------------------------------------------------------------- */

struct VoxelDrawNode;

impl bevy::render::render_graph::Node for VoxelDrawNode {
    fn run(
        &self,
        _g: &mut RenderGraphContext,
        ctx: &mut bevy::render::renderer::RenderContext,
        w:   &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        /* quick-out when nothing to draw */
        let slices = w.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() { return Ok(()); }

        /* pipeline ready? */
        let pipes = w.resource::<PipelineCache>();
        let vox   = w.resource::<VoxelPipeline>();
        let Some(pipe) = pipes.get_render_pipeline(vox.id) else { return Ok(()); };

        /* active Core3d camera (order == 2) */
        let target = w.iter_entities()
            .filter_map(|e| Some((e.get::<Camera>()?, e.get::<bevy::render::view::ViewTarget>()?)))
            .find(|(cam, _)| cam.is_active && cam.order == 2)
            .map(|(_, tgt)| tgt);
        let Some(target) = target else { return Ok(()); };

        /* render pass */
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("voxel_draw_pass"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(pipe);

        /* buffers */
        let mesh = w.resource::<MeshletBuffers>();

        // `Buffer::slice(..)` already returns the type `RenderPass::set_vertex_buffer`
        // needs – no dereference necessary.
        pass.set_vertex_buffer(0, *mesh.vertices.slice(..));
        pass.draw_indirect(&mesh.indirect, 0);

        Ok(())
    }
}

/* ------------------------------------------------------------------- */
/* STARTUP system: allocate MeshletBuffers                             */
/* ------------------------------------------------------------------- */

fn init_meshlet_buffers(
    mut cmds  : Commands,
    device: Res<RenderDevice>,
) {
    cmds.insert_resource(MeshletBuffers::new(&*device, MAX_VOXELS));
}

/* ------------------------------------------------------------------- */
/* compile & cache all internal compute pipelines                  */
/* ------------------------------------------------------------------- */
fn init_compute_pipelines(
    device : Res<RenderDevice>,
    mut cache   : ResMut<GpuPipelineCache>,
    mut shaders : ResMut<Assets<Shader>>,
    mut pipes   : ResMut<PipelineCache>,
) {
    /* Dual-Contouring (guaranteed path) */
    let dc_src = include_str!("../../../../../assets/shaders/automatoxel/dual_contour.wgsl");
    cache.get_or_create("dual_contour", dc_src, &mut pipes, &mut shaders, &*device);

    /* Mesh-shader fast path (only compiled when the feature is on) */
    #[cfg(feature = "mesh_shaders")]
    {
        let mp_src = include_str!("../../../../../assets/shaders/automatoxel/mesh_path.wgsl");
        cache.get_or_create("mesh_path", mp_src, &mut pipes, &mut shaders, &*device);
    }
}

/* ------------------------------------------------------------------- */
/* new render-world system                                             */
/* ------------------------------------------------------------------- */
#[cfg(feature = "gpu-debug")]
fn render_debug_squares(
    mut ev:     EventReader<DebugSeedSquare>,
    queue:      Res<bevy::render::renderer::RenderQueue>,
    images:     Res<bevy::render::render_asset::RenderAssets<
        bevy::render::texture::GpuImage,
    >>,
    atlas:      Res<crate::plugin::GlobalVoxelAtlas>,
) {
    for dbg in ev.read() {
        seed_debug_square(&queue, &images, &atlas, &dbg.slice, dbg.value);
    }
}

//////////////////////////////////////////////////////////////////
//                    Plugin implementation                     //
//////////////////////////////////////////////////////////////////
pub struct GpuAutomataComputePlugin;

impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        /* 0 ░ register events in the main world */
        app.add_event::<DebugSeedSquare>()
           .add_event::<SimulationStep>(); // ← NEW

        /* 1 ░ create global textures & allocator */
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let atlas  = images.add(make_atlas("ca.atlas"));
        let signal = images.add(make_image("ca.signal"));
        drop(images);

        app.insert_resource(GlobalVoxelAtlas { atlas, signal })
           .insert_resource(AtlasAllocator::default());

        /* 2 ░ allocate atlas slices for new automatons */
        app.add_systems(
            Update,
            |mut cmd:   Commands,
             mut atlas: ResMut<AtlasAllocator>,
             mut ev:    EventReader<AutomatonAdded>,
             mut reg:   ResMut<AutomataRegistry>,
             slice_q:   Query<&GpuGridSlice>| {
                for ev in ev.read() {
                    if slice_q.get(ev.entity).is_ok() { continue; }
                    let Some(info) = reg.get_mut(ev.id) else { continue };
                    let want = info.slice.size.max(UVec2::splat(256));

                    if let Some((layer, off)) = atlas.allocate(want) {
                        let slice = GpuGridSlice {
                            layer,
                            offset: off,
                            size:   want,
                            rule:   info.name.clone(),
                            rule_bits: 0,
                        };
                        cmd.entity(ev.entity).insert(slice.clone());
                        info.slice        = slice.clone();
                        info.world_offset = Vec3::new(off.x as f32, off.y as f32, 0.0)
                                            * info.voxel_size;
                    } else {
                        warn!("atlas full – cannot allocate {}", info.name);
                    }
                }
            },
        );

        /* 3 ░ reset allocator when returning to menu */
        app.add_systems(
            OnEnter(AppState::MainMenu),
            |mut atlas: ResMut<AtlasAllocator>| *atlas = AtlasAllocator::default(),
        );

        /* 4 ░ render-sub-app setup */
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_event::<DebugSeedSquare>()
            .add_event::<SimulationStep>()          // ← NEW
            .init_resource::<ExtractedGpuSlices>()
            .init_resource::<GpuPipelineCache>()
            .init_resource::<StepsThisFrame>()      // ← NEW
            .insert_resource(FrameParity::default())
            /* flip parity each extract */
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            /* extraction helpers */
            .add_systems(ExtractSchedule, (
                extract_gpu_slices,
                count_sim_steps,                          // ← NEW
            ))
            /* forward events */
            .add_systems(ExtractSchedule, (
                extract_events::<DebugSeedSquare>,
                extract_events::<SimulationStep>,
            ))
            /* graph/pipeline startup */
            .add_systems(
                Startup,
                (
                    init_meshlet_buffers,
                    init_voxel_pipeline,
                    init_compute_pipelines,
                ),
            )
            /* be sure the Queue set exists */
            .configure_sets(Render, (RenderSet::Queue,));

        /* 5 ░ wire render-graph nodes */
        let mut graph  = render_app.world_mut().resource_mut::<RenderGraph>();
        let core3d_sub = graph.get_sub_graph_mut(Core3d).unwrap();

        core3d_sub.add_node(AutomataStepLabel, ComputeAutomataNode);

        #[cfg(feature = "mesh_shaders")]
        {
            core3d_sub.add_node(MeshPathLabel, crate::compute::mesh_path::MeshPathNode);
            core3d_sub.add_node_edge(Node3d::StartMainPass, AutomataStepLabel);
            core3d_sub.add_node_edge(AutomataStepLabel, MeshPathLabel);
            core3d_sub.add_node_edge(MeshPathLabel, Node3d::EndMainPass);
        }

        #[cfg(not(feature = "mesh_shaders"))]
        {
            use bevy::core_pipeline::core_3d::graph::Node3d;
            use crate::plugin::labels::{DrawVoxelLabel, DualContourLabel};

            core3d_sub.add_node(DualContourLabel, crate::compute::dual_contour::DualContourNode);
            core3d_sub.add_node(DrawVoxelLabel, VoxelDrawNode);
            core3d_sub.add_node_edge(Node3d::StartMainPass, AutomataStepLabel);
            core3d_sub.add_node_edge(AutomataStepLabel, DualContourLabel);
            core3d_sub.add_node_edge(DualContourLabel, DrawVoxelLabel);
            core3d_sub.add_node_edge(DrawVoxelLabel, Node3d::EndMainPass);
        }

        /* 6 ░ optional GPU-debug helper in render world */
        #[cfg(feature = "gpu-debug")]
        render_app.add_systems(Render, render_debug_squares.after(RenderSet::Queue));
    }
}
