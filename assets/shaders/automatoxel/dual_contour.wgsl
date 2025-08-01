/* ===================================================================== */
/* WGSL shader source                                                    */
/* ===================================================================== */

/// **`dc.wgsl`** – Compute‑driven Dual Contouring kernel.
///
/// * Thread‑group size: `@workgroup_size(8,8,8)` – one invocation per
///   voxel cube.
/// * All integer maths (u32) – affordable even on mobile GPUs.
struct Vertex {
    pos : vec3f,
    nrm : vec3f,
    mat : u32,
};

@group(0) @binding(0) var<storage_texture_3d, read>  vox_ping  : texture_3d<u32>;
@group(0) @binding(1) var<storage_texture_3d, read>  vox_pong  : texture_3d<u32>;
@group(0) @binding(2) var<storage, read_write>       out_vert  : array<Vertex>;
@group(0) @binding(3) var<storage, read_write>       counter   : atomic<u32>;

@compute @workgroup_size(8,8,8)
fn main(@builtin(global_invocation_id) gid : vec3u) {
    // Fetch the eight voxel values (densities) – simplify: 0|1 binary field.
    let p = gid;
    var d : array<u32, 8>;
    let offsets = array<vec3u,8>(
        vec3u(0,0,0), vec3u(1,0,0), vec3u(1,1,0), vec3u(0,1,0),
        vec3u(0,0,1), vec3u(1,0,1), vec3u(1,1,1), vec3u(0,1,1));
    for (var i:u32 = 0u; i < 8u; i = i + 1u) {
        d[i] = textureLoad(vox_ping, (p + offsets[i]).xyz, 0).x & 1u;
    }
    // Simple early‑out: if all equal → no surface crosses this cell.
    let sum = d[0]+d[1]+d[2]+d[3]+d[4]+d[5]+d[6]+d[7];
    if (sum == 0u || sum == 8u) { return; }

    // *** Placeholder – full Dual Contouring Hermite solve omitted. ***
    // We write a dummy single triangle centred at the cube for now so
    // we can validate the data‑flow end‑to‑end.

    let v0 = Vertex(vec3f(f32(p.x)+0.0, f32(p.y)+0.0, f32(p.z)+0.0), vec3f(0.0,0.0,1.0), 0u);
    let v1 = Vertex(vec3f(f32(p.x)+1.0, f32(p.y)+0.0, f32(p.z)+0.0), vec3f(0.0,0.0,1.0), 0u);
    let v2 = Vertex(vec3f(f32(p.x)+0.0, f32(p.y)+1.0, f32(p.z)+0.0), vec3f(0.0,0.0,1.0), 0u);

    let base = atomicAdd(&counter, 3u);
    out_vert[base]     = v0;
    out_vert[base+1u]  = v1;
    out_vert[base+2u]  = v2;
}