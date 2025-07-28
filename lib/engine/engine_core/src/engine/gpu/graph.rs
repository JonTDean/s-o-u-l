//! GPU render‑graph compute node – Z‑layer batching with ping‑pong swap.

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};
use bevy::render::render_resource::*;
use bevy_egui::egui::ahash::HashMap;
use std::mem::size_of;

use super::{
    pipelines::GpuPipelineCache,
    plugin::{FrameParity, GlobalStateTextures},
    types::AutomatonParams,
};

/* ───────────── Data copied from main‑world to render‑world ───────────── */

#[derive(Component, Clone)]
pub struct GpuGridSlice {
    pub layer:     u32,
    pub offset:    UVec2,
    pub size:      UVec2,
    pub rule:      String,
    pub rule_bits: u32,
}

/// Mirror component that lives in the *render* world.
#[derive(Component, Clone)]
pub struct RenderGpuGridSlice(pub GpuGridSlice);

/// Resource holding every slice that must be stepped this frame.
#[derive(Resource, Default, Clone)]
pub struct ExtractedGpuSlices(pub Vec<RenderGpuGridSlice>);

/// Extract system: clone the component & fill the resource every frame.
pub fn extract_gpu_slices(
    mut cmd: Commands,
    q: Query<(Entity, &GpuGridSlice)>,
    mut slices_res: ResMut<ExtractedGpuSlices>,
) {
    slices_res.0.clear();
    for (e, slice) in &q {
        let copy = RenderGpuGridSlice(slice.clone());
        cmd.entity(e).insert(copy.clone());
        slices_res.0.push(copy);
    }
}

/* ─────────────────────────── render‑graph node ─────────────────────────── */

pub struct ComputeAutomataNode;

impl Node for ComputeAutomataNode {
    #[allow(clippy::too_many_lines)]
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_ctx: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        /* 1 ░ group slices by rule so every dispatch uses one pipeline */
        let batches: HashMap<String, Vec<RenderGpuGridSlice>> = {
            let mut map: HashMap<String, Vec<RenderGpuGridSlice>> = HashMap::default();
            for slice in &world.resource::<ExtractedGpuSlices>().0 {
                map.entry(slice.0.rule.clone())
                   .or_default()
                   .push(slice.clone());
            }
            map
        };

        /* 2 ░ gather shared GPU resources (immutable) */
        let device     = world.resource::<RenderDevice>();
        let queue      = world.resource::<RenderQueue>();
        let pipelines  = world.resource::<PipelineCache>();
        let cache      = world.resource::<GpuPipelineCache>();
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let textures   = world.resource::<GlobalStateTextures>();
        let parity_res = world.resource::<FrameParity>();

        /* 3 ░ start a compute pass */
        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("ca.compute_step"),
                ..default()
            });

        /* 4 ░ tiny 4‑byte buffer: current frame parity (ping / pong) */
        let parity_buf = device.create_buffer(&BufferDescriptor {
            label: Some("frame_parity"),
            size:  size_of::<u32>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&parity_buf, 0, bytemuck::bytes_of(&(parity_res.0 as u32)));

        /* 5 ░ iterate over every rule → one dispatch per rule */
        for (rule_id, slices) in batches {
            if slices.is_empty() { continue; }

            /* 5‑a ░ locate the cached compute pipeline */
            let Some(&pipe_id) = cache.map.get(&rule_id) else { continue };
            let Some(pipe)     = pipelines.get_compute_pipeline(pipe_id) else { continue };

            /* 5‑b ░ build the AutomatonParams SSBO */
            let params: Vec<AutomatonParams> = slices.iter().map(|s| AutomatonParams {
                size_x:   s.0.size.x,
                size_y:   s.0.size.y,
                layer:    s.0.layer,
                rule:     s.0.rule_bits,
                offset_x: s.0.offset.x,
                offset_y: s.0.offset.y,
            }).collect();

            let params_buf = device.create_buffer(&BufferDescriptor {
                label: Some("params_array"),
                size:  (params.len() * size_of::<AutomatonParams>()) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&params_buf, 0, bytemuck::cast_slice(&params));

            /* 5‑c ░ texture views */
            let ping_view   = &gpu_images.get(&textures.ping).unwrap().texture_view;
            let pong_view   = &gpu_images.get(&textures.pong).unwrap().texture_view;
            let signal_view = &gpu_images.get(&textures.signal).unwrap().texture_view;

            /* 5‑d ░ bind group */
            let bevy_layout =
                BindGroupLayout::from(pipe.get_bind_group_layout(0).clone());

            let bind_label = format!("ca.bind.{rule_id}");
            let bind_group = device.create_bind_group(
                Some(bind_label.as_str()),
                &bevy_layout,
                &[
                    BindGroupEntry { binding: 0, resource: BindingResource::TextureView(ping_view) },
                    BindGroupEntry { binding: 1, resource: BindingResource::TextureView(pong_view) },
                    BindGroupEntry { binding: 2, resource: parity_buf.as_entire_binding()        },
                    BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding()        },
                    BindGroupEntry { binding: 4, resource: BindingResource::TextureView(signal_view) },
                ],
            );

            /* 5‑e ░ dispatch */
            let max_w = slices.iter().map(|s| s.0.size.x).max().unwrap();
            let max_h = slices.iter().map(|s| s.0.size.y).max().unwrap();
            let wg_x  = (max_w + 15) / 16;
            let wg_y  = (max_h + 15) / 16;
            let wg_z  = slices.len() as u32;

            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, wg_z);
        }

        /* 6 ░ submit pass – parity will flip automatically in ExtractSchedule */
        drop(pass);
        Ok(())
    }
}
