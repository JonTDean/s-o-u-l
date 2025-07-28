use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;


#[derive(Resource, Default)]
pub struct GpuPipelineCache {
    /// `rule_id → cached pipeline`
    pub map: std::collections::HashMap<String, CachedComputePipelineId>,
}

impl GpuPipelineCache {
    pub fn get_or_create(
        &mut self,
        rule_id:      &str,
        shader_source:&str,
        pipelines:    &mut PipelineCache,
        shaders:      &mut Assets<Shader>,
        _device:      &RenderDevice,
    ) -> CachedComputePipelineId {
        if let Some(&id) = self.map.get(rule_id) { return id; }

        // ❶  make the source *owned*  →  `'static`  (fixes lifetime error)
        let shader_handle = shaders.add(Shader::from_wgsl(
            shader_source.to_owned(),              // <- to_owned()
            format!("compute::{rule_id}"),         // virtual “path” for nice error msgs
        ));

        let cs = ComputePipelineDescriptor {
            label:  Some(format!("pipeline::{rule_id}").into()),
            layout: vec![],                        // reflection will build it
            push_constant_ranges: vec![],
            shader: shader_handle,
            shader_defs: vec![],
            entry_point: Cow::Borrowed("main"),
            zero_initialize_workgroup_memory: false,
        };

        let id = pipelines.queue_compute_pipeline(cs);
        self.map.insert(rule_id.into(), id);
        id
    }
}
