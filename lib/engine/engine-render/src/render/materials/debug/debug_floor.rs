use bevy::{
    prelude::*, 
    render::{mesh:: MeshVertexBufferLayoutRef, render_resource::*}, 
    sprite::{AlphaMode2d, Material2d, Material2dKey}
};
use bevy::reflect::TypePath;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DebugFloorMaterial {
    #[uniform(0)]
    pub params: DebugFloorParams,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, ShaderType, Default)]
pub struct DebugFloorParams {
    pub zoom:   f32,
    pub alpha:  f32,
    pub _pad:   Vec2,
    pub origin: Vec2,
}

impl Material2d for DebugFloorMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/debug/debug_floor.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/debug/debug_floor.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout:     &MeshVertexBufferLayoutRef,
        _key:       Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
