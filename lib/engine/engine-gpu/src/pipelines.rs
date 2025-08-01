//! Lightweight **PipelineCache** wrapper.
//
//  Stores one entry per “rule-id” or per fixed pipeline so we can reuse
//  the compiled object across frames without hitting shader-compile
//  time-outs on slow drivers.

use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;

#[derive(Resource, Default, Clone)]
pub struct GpuPipelineCache {
    pub map: std::collections::HashMap<String, CachedComputePipelineId>,
}

impl GpuPipelineCache {
    pub fn get_or_create(
        &mut self,
        id:            &str,
        shader_source: &str,
        pipelines:     &mut PipelineCache,
        shaders:       &mut Assets<Shader>,
        _device:       &RenderDevice,
    ) -> CachedComputePipelineId {
        if let Some(&pid) = self.map.get(id) {
            return pid;
        }

        let handle = shaders.add(Shader::from_wgsl(shader_source.to_owned(), id));
        let desc = ComputePipelineDescriptor {
            label: Some(Cow::Owned(format!("pipeline::{id}"))),
            layout: vec![],
            push_constant_ranges: vec![],
            shader: handle,
            shader_defs: vec![],
            entry_point: Cow::Borrowed("main"),
            zero_initialize_workgroup_memory: false,
        };
        let pid = pipelines.queue_compute_pipeline(desc);
        self.map.insert(id.into(), pid);
        pid
    }
}
