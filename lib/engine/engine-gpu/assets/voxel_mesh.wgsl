// The first field matches Bevy ViewUniform::clip_from_world. The explicit
// bind layout retains Bevy's full minimum size and dynamic view offset.
struct View { clip_from_world: mat4x4<f32> };
@group(0) @binding(0) var<uniform> view: View;
struct VSOut {
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @builtin(position) pos: vec4<f32>,
};
@vertex
fn vertex(@location(0) pos: vec4<f32>, @location(1) normal: vec4<f32>) -> VSOut {
    var out: VSOut;
    out.pos = view.clip_from_world * pos;
    out.normal = normal.xyz;
    out.world_pos = pos.xyz;
    return out;
}
@fragment
fn fragment(in: VSOut) -> @location(0) vec4<f32> {
    let light = 0.35 + 0.65 * max(dot(normalize(in.normal), normalize(vec3<f32>(0.4, 0.8, 0.6))), 0.0);
    let band = 0.8 + 0.2 * sin(in.world_pos.y * 0.12);
    return vec4<f32>(vec3<f32>(0.12, 0.72, 0.62) * light * band, 1.0);
}
