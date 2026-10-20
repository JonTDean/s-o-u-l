// Continuous XYZ Lenia v1. State words carry f32 bits; only atlas presentation quantizes.
struct Params { size: vec4<u32>, growth: vec4<f32>, kernel: vec4<u32> }
@group(0) @binding(0) var<storage, read> previous: array<u32>;
@group(0) @binding(1) var<storage, read_write> next: array<u32>;
@group(0) @binding(2) var<storage, read_write> packed: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;
fn density(at: vec3<i32>) -> f32 {
    var p = at;
    let size = vec3<i32>(params.size.xyz);
    if params.kernel.z == 1u { p = ((p % size) + size) % size; }
    if any(p < vec3<i32>(0)) || any(p >= size) { return 0.0; }
    return bitcast<f32>(previous[(u32(p.z) * params.size.y + u32(p.y)) * params.size.x + u32(p.x)]);
}
@compute @workgroup_size(8,8,1)
fn evolve(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id >= params.size.xyz) { return; }
    let radius = i32(params.kernel.x);
    var sum = 0.0;
    var norm = 0.0;
    for (var z = -radius; z <= radius; z += 1) {
        for (var y = -radius; y <= radius; y += 1) {
            for (var x = -radius; x <= radius; x += 1) {
                let delta = vec3<i32>(x,y,z);
                let r = length(vec3<f32>(delta)) / f32(radius);
                if r <= 1.0 {
                    var mean = 0.0;
                    var width = 0.5;
                    if params.kernel.y == 1u { mean = 0.65; width = 0.2; }
                    let d = (r-mean)/width;
                    let weight = exp(-0.5*d*d);
                    norm += weight;
                    sum += weight * density(vec3<i32>(id)+delta);
                }
            }
        }
    }
    let d = (sum/norm-params.growth.x)/params.growth.y;
    let growth = 2.0*exp(-0.5*d*d)-1.0;
    let index = (id.z*params.size.y+id.y)*params.size.x+id.x;
    next[index] = bitcast<u32>(clamp(bitcast<f32>(previous[index])+params.growth.z*growth,0.0,1.0));
}
@compute @workgroup_size(64)
fn pack(@builtin(global_invocation_id) id: vec3<u32>) {
    let words = params.size.w/4u;
    if id.x >= words || id.y >= params.size.y || id.z >= params.size.z { return; }
    let row = id.z*params.size.y+id.y;
    var word = 0u;
    for (var b = 0u; b < 4u; b += 1u) {
        let x = id.x*4u+b;
        if x < params.size.x {
            let v = bitcast<f32>(previous[row*params.size.x+x]);
            word |= u32(floor(clamp(v,0.0,1.0)*255.0+0.5)) << (b*8u);
        }
    }
    packed[row*words+id.x] = word;
}
