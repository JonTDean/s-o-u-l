use bevy::render::render_graph::RenderLabel;

/// Node labels used by the Core3D sub-graph.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct AutomataStepLabel;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DualContourLabel;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct MeshPathLabel;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct MeshShaderLabel;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DrawVoxelLabel;
