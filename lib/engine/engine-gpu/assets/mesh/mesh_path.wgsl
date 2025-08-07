// Mesh-path compute shader – post-processes the dual-contour stream.
// group(0) bindings:
//  0 → input  vertex buffer  (read-only)
//  1 → output vertex buffer  (read / write)
//  2 → DrawIndirect struct   (std430)
//  3 → vertex counter atomics
//
// Binding 1 lets us route the stream into a *different* buffer if we ever
// want to compact, compress, or format-convert in place. For now both
// bindings point at the same buffer, so this kernel is an identity pass.

struct DrawArgs {
    vertex_count   : u32,
    instance_count : u32,
    first_vertex   : u32,
    first_instance : u32,
};

@group(0) @binding(0) var<storage, read>       vtx_in  : array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> vtx_out : array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> args    : DrawArgs;
@group(0) @binding(3) var<storage, read>       counter : atomic<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id : vec3<u32>) {
    let i = id.x;

    // Bail out when `i` exceeds the vertex count produced by dual-contour.
    if (i >= atomicLoad(&counter)) { return; }

    // Identity copy (replace with real post-processing later).
    vtx_out[i] = vtx_in[i];

    // One thread writes the indirect-draw header.
    if (i == 0u) {
        args.vertex_count   = atomicLoad(&counter);
        args.instance_count = 1u;
        args.first_vertex   = 0u;
        args.first_instance = 0u;
    }
}
