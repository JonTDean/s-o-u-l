//! Builds the graphics pipeline that uses TASK + MESH shaders.
#![cfg(feature = "mesh_shaders")]

/* ── imports ─────────────────────────────────────────────── */
use ash::vk;
use naga::{
    back::spv,
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};
use std::ffi::CStr;

use crate::ash::AshContext;

/* ── helper macro: C‑string literals for Ash ─────────────── */
macro_rules! cstr {
    ($s:expr) => {{
        const BYTES: &[u8] = concat!($s, "\0").as_bytes();
        // SAFETY: BYTES is NUL‑terminated and has no interior NULs.
        unsafe { CStr::from_bytes_with_nul_unchecked(BYTES) }
    }};
}

/* ------------------------------------------------------------------ */
pub fn create_mesh_pipeline(
    ctx:             &AshContext,
    render_pass:     vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
) -> vk::Pipeline {
    /* 1 · WGSL → SPIR‑V --------------------------------------------- */
    const MESH_SRC: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/mesh/mesh_topology.wgsl"
    ));
    const TASK_SRC: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/mesh/mesh_task.wgsl"
    ));

    let mesh_spv = wgsl_to_spirv(MESH_SRC,  "mesh_topology.wgsl");
    let task_spv = wgsl_to_spirv(TASK_SRC,  "mesh_task.wgsl");

    /* 2 · Create shader modules + pipeline --------------------------- */
    // Unsafe FFI boundary: Ash expects raw SPIR‑V to be valid.
    let mesh_module = unsafe { shader_module(&ctx.device, &mesh_spv) };
    let task_module = unsafe { shader_module(&ctx.device, &task_spv) };

    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MESH_EXT)
            .module(mesh_module)
            .name(cstr!("main")),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::TASK_EXT)
            .module(task_module)
            .name(cstr!("main")),
    ];
    let dyn_state = vk::PipelineDynamicStateCreateInfo::default();

    // Whole pipeline creation is also an unsafe Vulkan FFI call.
    let pipeline = unsafe {
        ctx.device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[vk::GraphicsPipelineCreateInfo::default()
                    .stages(&stages)
                    .layout(pipeline_layout)
                    .render_pass(render_pass)
                    .dynamic_state(&dyn_state)],
                None,
            )
            .expect("create mesh pipeline")[0]
    };

    // Shader modules can be freed right after pipeline creation.
    unsafe {
        ctx.device.destroy_shader_module(mesh_module, None);
        ctx.device.destroy_shader_module(task_module, None);
    }

    pipeline
}

/* ── helpers ─────────────────────────────────────────────── */
fn wgsl_to_spirv(source: &str, file_name: &str) -> Vec<u32> {
    let module = wgsl::parse_str(source).expect(file_name);
    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .expect("naga validation");
    spv::write_vec(&module, &info, &spv::Options::default(), None)
        .expect("write SPIR‑V")
}

/// Create a Vulkan shader module from raw SPIR‑V words.
///
/// # Safety
/// Caller must guarantee that `words` form a valid SPIR‑V module for
/// the target device.
unsafe fn shader_module(device: &ash::Device, words: &[u32]) -> vk::ShaderModule {
    unsafe {
        device
            .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(words), None)
            .expect("shader module")
    }
}
