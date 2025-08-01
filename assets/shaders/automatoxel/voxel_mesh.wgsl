// Vertex input matching the Vertex struct (pos and normal only; material id is ignored)
struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) nrm: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

const WORLD_SIZE : vec2<f32> = vec2(1024.0, 1024.0);

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Simple orthographic projection: map world coordinates [0, WORLD_SIZE] to NDC [-1, 1]
    let ndc_x = input.pos.x / (WORLD_SIZE.x * 0.5) - 1.0;
    let ndc_y = input.pos.y / (WORLD_SIZE.y * 0.5) - 1.0;
    out.clip_pos = vec4(ndc_x, ndc_y, 0.0, 1.0);

    // Color by normal (for debugging: transform normals from [-1,1] to [0,1] range)
    out.color = input.nrm * 0.5 + 0.5;
    return out;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Output the vertex color (with full opacity)
    return vec4(input.color, 1.0);
}
