// automatoxel/mesh_path.wgsl — 2 → 3 pass GPU pipeline (copy+indirect)
//
// * Work-group of **256** threads copies vertices from an append-only SSBO
//   into a regular vertex buffer and writes one `DrawIndirectArgs` struct
//   (matching D3D12/Vulkan layout) once per dispatch.
//
// * Follows WGSL atomic-memory semantics: only `atomicLoad` is required here
//   (SC-acquire) because the counter is written *before* this kernel begins.
//   See WGSL spec §4.3 “Atomics” for details. :contentReference[oaicite:5]{index=5}
//
// © 2025 Obaven ™

struct Vertex { pos: vec3f, nrm: vec3f, mat: u32 };

struct DrawArgs {
    vertex_count   : u32,
    instance_count : u32,
    base_vertex    : u32,
    base_instance  : u32,
};

@group(0) @binding(0) var<storage, read>       in_vert  : array<Vertex>;
@group(0) @binding(1) var<storage, read_write> out_vert : array<Vertex>;
@group(0) @binding(2) var<storage, read_write> args     : DrawArgs;
@group(0) @binding(3) var<storage, read>       counter  : atomic<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let total = atomicLoad(&counter);          // SC-acquire
    if (gid.x < total) {
        out_vert[gid.x] = in_vert[gid.x];
    }
    // first thread writes indirect draw — layout matches wgpu::util::DrawIndirectArgs
    if (gid.x == 0u) {
        args.vertex_count   = total;
        args.instance_count = 1u;
        args.base_vertex    = 0u;
        args.base_instance  = 0u;
    }
}
