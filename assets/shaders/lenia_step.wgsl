// Quantized XY Lenia with zero boundaries and independent Z planes.
struct Params { size: vec4<u32>, growth: vec4<f32> }
@group(0) @binding(0) var<storage, read> previous: array<u32>;
@group(0) @binding(1) var<storage, read_write> next: array<u32>;
@group(0) @binding(2) var<storage, read_write> packed: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;

fn density(x: i32, y: i32, z: u32) -> f32 {
    if x < 0 || y < 0 || x >= i32(params.size.x) || y >= i32(params.size.y) {
        return 0.0;
    }
    let index = (z * params.size.y + u32(y)) * params.size.x + u32(x);
    return f32(previous[index]) / 255.0;
}

@compute @workgroup_size(8, 8, 1)
fn evolve(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id >= params.size.xyz) { return; }
    var potential = 0.0;
    for (var dy = -1; dy <= 1; dy += 1) {
        for (var dx = -1; dx <= 1; dx += 1) {
            var weight = 0.09653235;
            if dx == 0 && dy == 0 { weight = 0.15915494; }
            if dx != 0 && dy != 0 { weight = 0.05854983; }
            potential += density(i32(id.x) + dx, i32(id.y) + dy, id.z) * weight;
        }
    }
    let difference = potential - params.growth.x;
    let growth = 2.0 * exp(-difference * difference / (2.0 * params.growth.y * params.growth.y)) - 1.0;
    let index = (id.z * params.size.y + id.y) * params.size.x + id.x;
    let value = clamp(f32(previous[index]) / 255.0 + params.growth.z * growth, 0.0, 1.0);
    // Explicit positive half-up rounding matches Rust f32::round.
    next[index] = u32(floor(value * 255.0 + 0.5));
}

// Pack four density bytes per word into 256-byte-aligned texture-copy rows.
@compute @workgroup_size(64)
fn pack(@builtin(global_invocation_id) id: vec3<u32>) {
    let words_per_row = params.size.w / 4u;
    if id.x >= words_per_row * params.size.y * params.size.z { return; }
    let x = (id.x % words_per_row) * 4u;
    let row = id.x / words_per_row;
    var word = 0u;
    for (var byte = 0u; byte < 4u; byte += 1u) {
        if x + byte < params.size.x {
            word |= previous[row * params.size.x + x + byte] << (byte * 8u);
        }
    }
    packed[id.x] = word;
}
