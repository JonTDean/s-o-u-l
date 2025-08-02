// --- Dual-Contouring (minimal) --------------------------------------------
// Reads an R8Uint density volume, emits up to 5 quads/voxel into an
// append-buffer.  ONE UINT per voxel: 0 = empty, 255 = solid.

struct Vertex {
    pos : vec3<f32>,
    nrm : vec3<f32>,
    mat : u32,
};

@group(0) @binding(0) var<storage, read_write> atlas : texture_storage_3d<r8uint, read_write>;
@group(0) @binding(1) var<storage, read_write> vertices : array<Vertex>;
@group(0) @binding(2) var<storage, read_write> counter  : atomic<u32>;

fn cube_corners(i : vec3<u32>) -> array<vec3<u32>, 8u> {
    return array<vec3<u32>, 8u>(
        i + vec3<u32>(0u,0u,0u), i + vec3<u32>(1u,0u,0u),
        i + vec3<u32>(1u,1u,0u), i + vec3<u32>(0u,1u,0u),
        i + vec3<u32>(0u,0u,1u), i + vec3<u32>(1u,0u,1u),
        i + vec3<u32>(1u,1u,1u), i + vec3<u32>(0u,1u,1u));
}

@compute @workgroup_size(8,8,8)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    // AABB: atlas.size.xyz – inject via push-constants in a later pass.
    if textureDimensions(atlas).x <= gid.x + 1u ||
       textureDimensions(atlas).y <= gid.y + 1u ||
       textureDimensions(atlas).z <= gid.z + 1u {
        return;
    }

    // --- 1 · sample the eight voxel corners ------------------------
    let corners = cube_corners(gid);
    var mask : u32 = 0u;
    for (var i = 0u; i < 8u; i = i + 1u) {
        let d = textureLoad(atlas, corners[i], 0).r;
        if d > 127u { mask = mask | (1u << i); }
    }

    // empty or full cube → nothing to contour
    if mask == 0u || mask == 0xffu { return; }

    // --- 2 · generate ONE quad centred on the voxel (demo only) ----
    let base  = atomicAdd(&counter, 6u);
    let centre = vec3<f32>(gid) + vec3<f32>(0.5);
    let verts = array<vec3<f32>, 6u>(
        centre + vec3<f32>(-0.5, -0.5, 0.0),
        centre + vec3<f32>( 0.5, -0.5, 0.0),
        centre + vec3<f32>( 0.5,  0.5, 0.0),
        centre + vec3<f32>(-0.5, -0.5, 0.0),
        centre + vec3<f32>( 0.5,  0.5, 0.0),
        centre + vec3<f32>(-0.5,  0.5, 0.0));

    for (var i = 0u; i < 6u; i = i + 1u) {
        vertices[base + i].pos = verts[i];
        vertices[base + i].nrm = vec3<f32>(0.0, 0.0, 1.0);
        vertices[base + i].mat = 0u;
    }
}
