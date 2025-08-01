//! Builds one **graphics pipeline** that consumes TASK + MESH shaders.
//!
//! The WGSL kernels live in `assets/shaders/automatoxel/mesh_{task,mesh}.wgsl`.
//! They are compiled to SPIR-V at run-time through *Naga-Oil* so hot-reload
//! in Bevy’s asset-pipeline keeps working even though mesh-shader stages
//! are still experimental in WGPU.
//!
//! ────────────────────────────────────────────────────────────────
//! © 2025 Obaven Inc. — Apache-2.0 OR MIT
//

use ash::{vk, Device};
use naga_oil::{back::spv, compose::Composer};

use crate::ash::AshContext;

/// Compile WGSL → SPIR-V and create a mesh-shader pipeline.
/// Returns the finished `vk::Pipeline`.
pub fn create_mesh_pipeline(
    ctx:             &AshContext,
    render_pass:     vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
) -> vk::Pipeline {
    // ── 1 · WGSL → SPIR-V ─────────────────────────────────────────────
    let mut comp = Composer::default();
    let mesh_spv = compile_wgsl(
        &mut comp,
        include_str!("../../../assets/shaders/automatoxel/mesh_path.wgsl"),
    );
    let task_spv = compile_wgsl(
        &mut comp,
        include_str!("../../../assets/shaders/automatoxel/mesh_task.wgsl"),
    );

    unsafe {
        // Shader modules
        let mesh_mod = create_module(&ctx.device, &mesh_spv);
        let task_mod = create_module(&ctx.device, &task_spv);

        // Stage array
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::MESH_EXT)
                .module(mesh_mod)
                .name(cstr!("main")),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::TASK_EXT)
                .module(task_mod)
                .name(cstr!("main")),
        ];

        // Mesh-shader specific create-info (still ‘experimental’ in vk-headers)
        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .dynamic_state(&vk::PipelineDynamicStateCreateInfo::default());

        ctx.device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
            .expect("mesh pipeline")[0]
    }
}

// ────────────────────────────────────────────────────────────────────
// helpers
// ────────────────────────────────────────────────────────────────────
fn compile_wgsl(comp: &mut Composer, src: &str) -> Vec<u32> {
    let module = comp.make_naga_module(src).expect("WGSL → IR");
    spv::write_vec(&module, &spv::Options::default()).expect("IR → SPIR-V")
}

unsafe fn create_module(device: &Device, words: &[u32]) -> vk::ShaderModule {
    let info = vk::ShaderModuleCreateInfo::default().code(words);
    device.create_shader_module(&info, None).expect("shader module")
}

// Tiny macro for NUL-terminated entry-point names.
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}
