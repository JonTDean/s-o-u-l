use bevy::render::render_graph::RenderLabel;

/// Node labels used inside the Core3d graph.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct AutomataStepLabel;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DualContourLabel;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DrawVoxelLabel;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct MeshPathLabel;
