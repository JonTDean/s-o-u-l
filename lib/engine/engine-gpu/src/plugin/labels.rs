//! Execution order labels for the portable Core3D voxel graph.
use bevy::render::render_graph::RenderLabel;

/// Fixed-tick automaton compute dispatch.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct AutomataStepLabel;

/// Extract exposed voxel faces into the bounded vertex stream.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct VoxelSurfaceLabel;

/// Draw generated triangles into the current 3D camera view.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DrawVoxelLabel;
