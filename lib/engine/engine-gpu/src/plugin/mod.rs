//! GPU back-end bootstrap for the voxel-automata renderer.
//!
//! The plug-in wires a small render-graph around Bevy’s `Core2d` graph,
//! allocates the global ping-/pong-atlas, and schedules the compute & draw
//! nodes for **every** frame.  This revised version **delays** creation of
//! [`MeshletBuffers`] until a dedicated **startup system** runs – fixing the
//! panic caused by an early `RenderDevice` lookup before the render back-end
//! finished initialising.
//!
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT

use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{render_graph::RenderGraph, render_resource::*, ExtractSchedule, RenderApp},
};
use engine_core::{events::AutomatonAdded, prelude::AppState};

pub mod atlas;
pub mod labels;
pub mod textures;

use atlas::AtlasAllocator;
use labels::{AutomataStepLabel, DrawVoxelLabel, DualContourLabel};
#[cfg(feature = "mesh_shaders")]
use labels::MeshPathLabel;
use textures::make_image;

use crate::{
    compute::dual_contour::{MeshletBuffers, MAX_VOXELS},
    graph::{extract_gpu_slices, ComputeAutomataNode, ExtractedGpuSlices, GpuGridSlice},
    pipelines::GpuPipelineCache,
};

/* ──────────────────────────────────────────────────────────────────── */
/* Shared textures & global flags                                      */
/* ──────────────────────────────────────────────────────────────────── */
#[derive(Resource, Clone)]
pub struct GlobalStateTextures {
    pub ping: Handle<Image>,
    pub pong: Handle<Image>,
    pub signal: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct FrameParity(pub bool);

fn init_voxel_pipeline(
    mut cmds     : Commands,
    mut shaders  : ResMut<Assets<Shader>>,
    mut pipelines: ResMut<PipelineCache>,
) {
    let vp = VoxelPipeline::new(&mut *shaders, &mut *pipelines);
    cmds.insert_resource(vp);
}

/* ──────────────────────────────────────────────────────────────────── */
/* Compact voxel mesh pipeline                                         */
/* ──────────────────────────────────────────────────────────────────── */
#[derive(Resource)]
pub struct VoxelPipeline {
    pub id: CachedRenderPipelineId,
}

impl VoxelPipeline {
    fn new(
        shaders:   &mut Assets<Shader>,
        pipelines: &mut PipelineCache,
    ) -> Self {
        /* 1 ░ WGSL shader */
        let shader = shaders.add(Shader::from_wgsl(
            include_str!("../../../../../assets/shaders/automatoxel/voxel_mesh.wgsl"),
            "voxel_mesh",
        ));

        /* 2 ░ Pipeline descriptor */
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

        let id = pipelines.queue_render_pipeline(desc);
        Self { id }
    }
}

/* ──────────────────────────────────────────────────────────────────── */
/* Draw node – consumes the vertex & indirect buffers                  */
/* ──────────────────────────────────────────────────────────────────── */
struct VoxelDrawNode;

impl bevy::render::render_graph::Node for VoxelDrawNode {
    fn run(
        &self,
        _g: &mut bevy::render::render_graph::RenderGraphContext,
        ctx: &mut bevy::render::renderer::RenderContext,
        w: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        /* 0 ░ Quick-out if nothing to draw */
        let slices = w.resource::<ExtractedGpuSlices>();
        if slices.0.is_empty() {
            return Ok(());
        }

        /* 1 ░ Pipeline ready? */
        let pipelines = w.resource::<PipelineCache>();
        let vox = w.resource::<VoxelPipeline>();
        let Some(pipeline) = pipelines.get_render_pipeline(vox.id) else { return Ok(()); };

        /* 2 ░ Active 2-D camera (order == 2) */
        let target = w
            .iter_entities()
            .filter_map(|ent| Some((ent.get::<Camera>()?, ent.get::<bevy::render::view::ViewTarget>()?)))
            .find(|(cam, _)| cam.is_active && cam.order == 2)
            .map(|(_, tgt)| tgt);
        let Some(target) = target else { return Ok(()); };

        /* 3 ░ Render-pass */
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("voxel_draw_pass"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);

        /* 4 ░ Buffers */
        let mesh = w.resource::<MeshletBuffers>();
        pass.set_vertex_buffer(0, *mesh.vertices.slice(..));
        pass.draw_indirect(&mesh.indirect, 0);

        Ok(())
    }
}

/* ──────────────────────────────────────────────────────────────────── */
/*  STARTUP SYSTEM: create MeshletBuffers                               */
/* ──────────────────────────────────────────────────────────────────── */
/// Allocates the per-frame scratch buffers **after** `RenderDevice` is
/// available.  Runs exactly once during the render-app’s startup stage.
fn init_meshlet_buffers(mut commands: Commands, device: Res<bevy::render::renderer::RenderDevice>) {
    commands.insert_resource(MeshletBuffers::new(&*device, MAX_VOXELS));
}

/* ──────────────────────────────────────────────────────────────────── */
/* Main plug-in                                                         */
/* ──────────────────────────────────────────────────────────────────── */
pub struct GpuAutomataComputePlugin;

impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        /* 1 ░ Atlas textures (main world) */
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let ping = images.add(make_image("ca.ping"));
        let pong = images.add(make_image("ca.pong"));
        let signal = images.add(make_image("ca.signal"));
        drop(images);

        let global = GlobalStateTextures { ping, pong, signal };
        app.insert_resource(global.clone())
            .insert_resource(AtlasAllocator::default());

        /* 2 ░ Allocate slice on automaton spawn */
        app.add_systems(
            Update,
            |mut cmd: Commands, mut atlas: ResMut<AtlasAllocator>, mut ev: EventReader<AutomatonAdded>| {
                for ev in ev.read() {
                    let size = UVec2::splat(256);
                    if let Some((layer, off)) = atlas.allocate(size) {
                        cmd.entity(ev.entity).insert(GpuGridSlice {
                            layer,
                            offset: off,
                            size,
                            rule: "life".into(),
                            rule_bits: 0b0001_1000,
                        });
                    } else {
                        warn!("🏳️  atlas full — cannot allocate GPU slice");
                    }
                }
            },
        );

        /* 3 ░ Reset atlas when entering main-menu */
        app.add_systems(OnEnter(AppState::MainMenu), |mut atlas: ResMut<AtlasAllocator>| *atlas = AtlasAllocator::default());

        /* 4 ░ Render-app setup */
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(global)
            .init_resource::<ExtractedGpuSlices>()
            .init_resource::<GpuPipelineCache>()
            .insert_resource(FrameParity::default())
            // toggle parity each frame (cheap RNG replacement)
            .add_systems(ExtractSchedule, |mut p: ResMut<FrameParity>| p.0 = !p.0)
            .add_systems(ExtractSchedule, extract_gpu_slices)
            // *** NEW: allocate MeshletBuffers once RenderDevice exists ***
            .add_systems(Startup, init_meshlet_buffers)
            .add_systems(Startup, init_voxel_pipeline);

        /* 5 ░ Wire nodes into Core2d */
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        let core2d_sub = graph.get_sub_graph_mut(Core2d).unwrap();
        core2d_sub.add_node(AutomataStepLabel, ComputeAutomataNode);

        #[cfg(feature = "mesh_shaders")]
        {
            core2d_sub.add_node(MeshPathLabel, crate::compute::mesh_path::MeshPathNode);
            core2d_sub.add_node_edge(Node2d::StartMainPass, AutomataStepLabel);
            core2d_sub.add_node_edge(AutomataStepLabel, MeshPathLabel);
            core2d_sub.add_node_edge(MeshPathLabel, Node2d::EndMainPass);
        }

        #[cfg(not(feature = "mesh_shaders"))]
        {
            core2d_sub.add_node(DualContourLabel, crate::compute::dual_contour::DualContourNode);
            core2d_sub.add_node(DrawVoxelLabel, VoxelDrawNode);
            core2d_sub.add_node_edge(Node2d::StartMainPass, AutomataStepLabel);
            core2d_sub.add_node_edge(AutomataStepLabel, DualContourLabel);
            core2d_sub.add_node_edge(DualContourLabel, DrawVoxelLabel);
            core2d_sub.add_node_edge(DrawVoxelLabel, Node2d::EndMainPass);
        }
    }
}