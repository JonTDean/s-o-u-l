//! Render-graph node that steps **all automata slices** in parallel.

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};
use bevy_egui::egui::ahash::HashMap;
use std::mem::size_of;

use crate::{
    pipelines::GpuPipelineCache,
    plugin::{FrameParity, GlobalStateTextures},
    types::AutomatonParams,
};

/* ─── extracted slice meta (main → render world) ─────────────── */
#[derive(Component, Clone)] pub struct GpuGridSlice { pub layer: u32, pub offset: UVec2, pub size: UVec2, pub rule: String, pub rule_bits: u32 }
#[derive(Component, Clone)] pub struct RenderGpuGridSlice(pub GpuGridSlice);
#[derive(Resource, Default, Clone)] pub struct ExtractedGpuSlices(pub Vec<RenderGpuGridSlice>);

pub fn extract_gpu_slices(
    mut cmd:   Commands,
    query:     Query<(Entity, &GpuGridSlice)>,
    mut out:   ResMut<ExtractedGpuSlices>,
) {
    out.0.clear();
    for (e, slice) in &query {
        let copy = RenderGpuGridSlice(slice.clone());
        cmd.entity(e).insert(copy.clone());
        out.0.push(copy);
    }
}

/* ─── actual node ────────────────────────────────────────────── */
pub struct ComputeAutomataNode;

impl Node for ComputeAutomataNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        ctx:    &mut RenderContext,
        world:  &World,
    ) -> Result<(), NodeRunError> {
        /* ───────────── EARLY-OUT GUARD ───────────── */
        let (Some(device), Some(queue)) = (
            world.get_resource::<RenderDevice>(),
            world.get_resource::<RenderQueue>(),
        ) else { return Ok(()); };
        /* ─────────────────────────────────────────── */

        /* batch slices by rule */
        let mut batches: HashMap<String, Vec<RenderGpuGridSlice>> = HashMap::default();
        for slice in &world.resource::<ExtractedGpuSlices>().0 {
            batches.entry(slice.0.rule.clone()).or_default().push(slice.clone());
        }
        if batches.is_empty() {
            return Ok(());
        }

        /* resources */
        let cache   = world.resource::<GpuPipelineCache>();
        let pipes   = world.resource::<PipelineCache>();
        let images  = world.resource::<RenderAssets<GpuImage>>();
        let tex     = world.resource::<GlobalStateTextures>();
        let parity  = world.resource::<FrameParity>();

        /* frame-parity uniform */
        let parity_buf = device.create_buffer(&BufferDescriptor {
            label: Some("frame_parity"),
            size:  4,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&parity_buf, 0, bytemuck::bytes_of(&(parity.0 as u32)));

        /* compute pass */
        let mut pass = ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("automata_step.compute"),
            timestamp_writes: None,
        });

        for (rule_id, slices) in batches {
            let Some(&pid) = cache.map.get(&rule_id) else { continue };
            let Some(pipe) = pipes.get_compute_pipeline(pid) else { continue };

            /* params SSBO */
            let params: Vec<AutomatonParams> = slices
                .iter()
                .map(|s| AutomatonParams {
                    size_x:   s.0.size.x,
                    size_y:   s.0.size.y,
                    layer:    s.0.layer,
                    rule:     s.0.rule_bits,
                    offset_x: s.0.offset.x,
                    offset_y: s.0.offset.y,
                })
                .collect();

            let params_buf = device.create_buffer(&BufferDescriptor {
                label: Some("params_array"),
                size:  (params.len() * size_of::<AutomatonParams>()) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&params_buf, 0, bytemuck::cast_slice(&params));

            /* bind-group */
            let layout = BindGroupLayout::from(pipe.get_bind_group_layout(0));
            let entries = [
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &images.get(&tex.ping).unwrap().texture_view,
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &images.get(&tex.pong).unwrap().texture_view,
                    ),
                },
                BindGroupEntry { binding: 2, resource: parity_buf.as_entire_binding() },
                BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding() },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(
                        &images.get(&tex.signal).unwrap().texture_view,
                    ),
                },
            ];

            let bind = device.create_bind_group(Some("automata_step.bind0"), &layout, &entries);

            /* dispatch */
            let w  = slices.iter().map(|s| s.0.size.x).max().unwrap();
            let h  = slices.iter().map(|s| s.0.size.y).max().unwrap();
            let gx = (w + 15) / 16;
            let gy = (h + 15) / 16;
            let gz = slices.len() as u32;

            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(gx, gy, gz);
        }
        Ok(())
    }
}
