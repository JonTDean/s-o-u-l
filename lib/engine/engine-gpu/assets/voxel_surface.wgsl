// Portable block-voxel surface extraction. Each occupied cell emits only
// faces adjacent to empty cells within its own atlas region.
struct Vertex { pos: vec4<f32>, nrm: vec4<f32> };
struct Params { origin: vec4<u32>, size: vec4<u32>, world: vec4<f32> };
struct DrawArgs { vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32 };
@group(0) @binding(0) var atlas: texture_3d<u32>;
@group(0) @binding(1) var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(2) var<storage, read_write> counter: atomic<u32>;
@group(0) @binding(3) var<uniform> params: Params;
@group(0) @binding(4) var<storage, read_write> draw: DrawArgs;

fn occupied(local: vec3<i32>) -> bool {
    if any(local < vec3<i32>(0)) || any(local >= vec3<i32>(params.size.xyz)) { return false; }
    let position = local + vec3<i32>(params.origin.xyz);
    if any(position >= vec3<i32>(textureDimensions(atlas))) { return false; }
    return textureLoad(atlas, position, 0).r > 127u;
}

// Explicit face bases avoid runtime indexing of constant vector arrays in
// SPIR-V, which is unreliable on some driver shader compilers.
fn emit_face(id: vec3<u32>, normal_i: vec3<i32>, u: vec3<f32>, v: vec3<f32>) {
    if occupied(vec3<i32>(id) + normal_i) { return; }
    let base = atomicAdd(&counter, 6u);
    if base + 6u > arrayLength(&vertices) { return; }
    let normal = vec4<f32>(vec3<f32>(normal_i), 0.0);
    let centre = vec3<f32>(id) + vec3<f32>(0.5) + normal.xyz * 0.5;
    let a = vec4<f32>(params.world.xyz + (centre - u*0.5 - v*0.5)*params.world.w, 1.0);
    let b = vec4<f32>(params.world.xyz + (centre + u*0.5 - v*0.5)*params.world.w, 1.0);
    let c = vec4<f32>(params.world.xyz + (centre + u*0.5 + v*0.5)*params.world.w, 1.0);
    let d = vec4<f32>(params.world.xyz + (centre - u*0.5 + v*0.5)*params.world.w, 1.0);
    vertices[base] = Vertex(a, normal);
    vertices[base+1u] = Vertex(b, normal);
    vertices[base+2u] = Vertex(c, normal);
    vertices[base+3u] = Vertex(a, normal);
    vertices[base+4u] = Vertex(c, normal);
    vertices[base+5u] = Vertex(d, normal);
}

@compute @workgroup_size(4, 4, 4)
fn surface(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id >= params.size.xyz) || !occupied(vec3<i32>(id)) { return; }
    emit_face(id, vec3<i32>(1,0,0), vec3<f32>(0,1,0), vec3<f32>(0,0,1));
    emit_face(id, vec3<i32>(-1,0,0), vec3<f32>(0,0,1), vec3<f32>(0,1,0));
    emit_face(id, vec3<i32>(0,1,0), vec3<f32>(0,0,1), vec3<f32>(1,0,0));
    emit_face(id, vec3<i32>(0,-1,0), vec3<f32>(1,0,0), vec3<f32>(0,0,1));
    emit_face(id, vec3<i32>(0,0,1), vec3<f32>(1,0,0), vec3<f32>(0,1,0));
    emit_face(id, vec3<i32>(0,0,-1), vec3<f32>(0,1,0), vec3<f32>(1,0,0));
}

// A separate dispatch orders all atomic appends before the indirect count.
@compute @workgroup_size(1)
fn finalize() {
    draw.vertex_count = min(atomicLoad(&counter), (arrayLength(&vertices) / 6u) * 6u);
    draw.instance_count = 1u;
    draw.first_vertex = 0u;
    draw.first_instance = 0u;
}
