// ─────────────────────────────────────────────────────────────────────────────
// Dual-Contouring (minimal “up to one quad per active voxel” prototype)
//
// Writes vertices in a **tight 32-byte layout**
//     struct Vertex { vec4 pos; vec4 nrm; };
//
// © 2025 Obaven Inc. — Apache-2.0 OR MIT
// ─────────────────────────────────────────────────────────────────────────────
struct Vertex {
    pos : vec4<f32>,
    nrm : vec4<f32>,
};

@group(0) @binding(0)             // 3-D atlas – integer values 0‥255
var atlas : texture_3d<u32>;                // read-only, use `textureLoad`
@group(0) @binding(1)             // vertex stream (std430, 32 B stride)
var<storage, read_write> vertices : @stride(32) array<Vertex>;
@group(0) @binding(2)             // atomic vertex counter
var<storage, read_write> counter  : atomic<u32>;

fn cube_corners(i : vec3<u32>) -> array<vec3<u32>, 8u> {
    return array<vec3<u32>, 8u>(
        i + vec3<u32>(0,0,0), i + vec3<u32>(1,0,0),
        i + vec3<u32>(1,1,0), i + vec3<u32>(0,1,0),
        i + vec3<u32>(0,0,1), i + vec3<u32>(1,0,1),
        i + vec3<u32>(1,1,1), i + vec3<u32>(0,1,1));
}

@compute @workgroup_size(8,8,8)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let size = textureDimensions(atlas);
    if any(gid + vec3<u32>(1) >= size) { return; }

    /* ── 1 · occupancy mask ─────────────────────────────────────── */
    var mask : u32 = 0u;
    let corners = cube_corners(gid);
    for (var i = 0u; i < 8u; i = i + 1u) {
        if textureLoad(atlas, corners[i], 0).r > 127u {
            mask = mask | (1u << i);
        }
    }
    if mask == 0u || mask == 0xffu { return; }        // empty or full voxel

    /* ── 2 · emit one demo quad (two tris) centred on the voxel ─── */
    let base   = atomicAdd(&counter, 6u);
    let centre = vec3<f32>(gid) + 0.5;

    let verts = array<vec3<f32>, 6u>(
        centre + vec3<f32>(-0.5, -0.5, 0.0),
        centre + vec3<f32>( 0.5, -0.5, 0.0),
        centre + vec3<f32>( 0.5,  0.5, 0.0),
        centre + vec3<f32>(-0.5, -0.5, 0.0),
        centre + vec3<f32>( 0.5,  0.5, 0.0),
        centre + vec3<f32>(-0.5,  0.5, 0.0));

    for (var i = 0u; i < 6u; i = i + 1u) {
        vertices[base + i].pos = vec4<f32>(verts[i], 1.0);
        vertices[base + i].nrm = vec4<f32>(0.0, 0.0, 1.0, 0.0);
    }
}
