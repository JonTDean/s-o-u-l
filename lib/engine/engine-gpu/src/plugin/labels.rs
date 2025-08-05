use bevy::render::render_graph::RenderLabel;

/// Node label for the automata compute step.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct AutomataStepLabel;
/// Node label for the dual-contouring stage.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DualContourLabel;
/// Node label for drawing voxel debug geometry.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct DrawVoxelLabel;
/// Node label for the optional mesh-shader path.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, RenderLabel)]
pub struct MeshPathLabel;
