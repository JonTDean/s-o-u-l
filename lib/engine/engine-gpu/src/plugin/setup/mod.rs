//! Renderer registration and main-to-render-world simulation statistics.
use crate::{
    compute::voxel::VoxelSurfaceNode,
    plugin::{StepsThisFrame, labels::*},
};
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        Extract,
        render_graph::{RenderGraphApp, ViewNodeRunner},
    },
};
use engine_core::systems::simulation::SimulationStep;

pub(super) mod allocation;
pub(super) mod seeding;

pub(super) fn count_sim_steps_main(
    mut events: EventReader<SimulationStep>,
    mut stats: ResMut<StepsThisFrame>,
) {
    stats.steps = events.read().count() as u32;
}

pub(super) fn extract_steps_this_frame(
    steps: Extract<Res<StepsThisFrame>>,
    mut commands: Commands,
) {
    commands.insert_resource(**steps);
}

pub(super) fn register_graph(app: &mut SubApp) {
    app.add_render_graph_node::<crate::graph::ComputeAutomataNode>(Core3d, AutomataStepLabel)
        .add_render_graph_node::<VoxelSurfaceNode>(Core3d, VoxelSurfaceLabel)
        .add_render_graph_node::<ViewNodeRunner<super::drawing::voxel_draw_node::VoxelDrawNode>>(
            Core3d,
            DrawVoxelLabel,
        )
        .add_render_graph_edges(
            Core3d,
            (
                Node3d::StartMainPass,
                AutomataStepLabel,
                VoxelSurfaceLabel,
                Node3d::MainOpaquePass,
                DrawVoxelLabel,
                Node3d::MainTransparentPass,
            ),
        );
}

#[cfg(feature = "gpu-debug")]
pub(super) fn extract_debug_seeds(
    events: Extract<Res<Events<engine_core::events::DebugSeedSquare>>>,
    mut cursor: Local<bevy::ecs::event::EventCursor<engine_core::events::DebugSeedSquare>>,
    mut pending: ResMut<super::resources::PendingDebugSeeds>,
) {
    pending.0.extend(cursor.read(&events).cloned());
}
