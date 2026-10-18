use super::*;
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    renderer::RenderContext,
};

#[derive(Default)]
pub(crate) struct LeniaStepNode;

impl Node for LeniaStepNode {
    fn run(
        &self,
        _: &mut RenderGraphContext,
        context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        super::execute::run(context, world);
        Ok(())
    }
}
