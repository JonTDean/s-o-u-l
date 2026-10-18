//! GPU voxel rendering, atlas allocation, and fixed-tick compute integration.
use bevy::{
    prelude::*,
    render::{ExtractSchedule, Render, RenderApp, RenderSet},
};
use engine_core::{
    events::{AutomatonAdded, AutomatonRemoved, DebugSeedSquare},
    prelude::AppState,
};

mod drawing;
pub mod labels;
mod resources;
mod setup;
pub mod storage;

pub(crate) use drawing::pipeline::VoxelPipeline;
pub use resources::{FrameParity, GlobalVoxelAtlas, StepsThisFrame, VoxelRenderStatus};
pub use storage::{atlas, textures};

/// Installs the portable voxel surface renderer and GPU atlas.
pub struct GpuAutomataComputePlugin;

impl Plugin for GpuAutomataComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AutomatonAdded>()
            .add_event::<AutomatonRemoved>()
            .add_event::<DebugSeedSquare>()
            .init_resource::<StepsThisFrame>()
            .init_resource::<VoxelRenderStatus>()
            .add_systems(Update, setup::count_sim_steps_main)
            .add_systems(Update, setup::ticks::request_ticks)
            .add_systems(Update, setup::allocation::allocate_slices)
            .add_systems(OnEnter(AppState::MainMenu), setup::allocation::reset_atlas);
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let atlas = images.add(textures::make_atlas("ca.atlas"));
        let signal = images.add(textures::make_image("ca.signal"));
        app.insert_resource(GlobalVoxelAtlas {
            atlas: atlas.clone(),
            signal: signal.clone(),
        })
        .init_resource::<atlas::AtlasAllocator>();
        let shader = drawing::pipeline::VoxelShader::register(app.world_mut());
        let status = app.world().resource::<VoxelRenderStatus>().clone();
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(shader).insert_resource(status);
        render_app
            .insert_resource(GlobalVoxelAtlas { atlas, signal })
            .init_resource::<crate::graph::ExtractedGpuSlices>()
            .init_resource::<crate::pipelines::GpuPipelineCache>()
            .init_resource::<StepsThisFrame>()
            .init_resource::<FrameParity>()
            .init_resource::<setup::seeding::SeededSlices>()
            .add_systems(
                ExtractSchedule,
                (
                    crate::graph::extract_gpu_slices,
                    setup::extract_steps_this_frame,
                ),
            )
            .add_systems(
                Render,
                setup::seeding::seed_new_slices.in_set(RenderSet::PrepareResources),
            );
        #[cfg(feature = "gpu-debug")]
        render_app
            .init_resource::<resources::PendingDebugSeeds>()
            .add_systems(ExtractSchedule, setup::extract_debug_seeds)
            .add_systems(
                Render,
                setup::seeding::apply_debug_seeds
                    .in_set(RenderSet::PrepareResources)
                    .after(setup::seeding::seed_new_slices),
            );
        setup::register_graph(render_app);
    }

    fn finish(&self, app: &mut App) {
        let shader = drawing::pipeline::VoxelShader::register(app.world_mut());
        let status = app.world().resource::<VoxelRenderStatus>().clone();
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(shader).insert_resource(status);
        render_app
            .init_resource::<crate::compute::voxel::MeshletBuffers>()
            .init_resource::<crate::compute::voxel::SurfacePipelines>()
            .init_resource::<VoxelPipeline>();
    }
}
