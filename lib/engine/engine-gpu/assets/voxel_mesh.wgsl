// Debug wire-frame vertex + fragment shader pair.
//
// Adds the camera view-projection matrix (group 0 / binding 0) so the mesh
// is rendered in the correct place on-screen.

struct View { view_proj : mat4x4<f32>; };

@group(0) @binding(0)
var<uniform> view : View;

struct VSOut {
    @location(0) world_pos : vec3<f32>,
    @builtin(position) pos : vec4<f32>,
};

@vertex
fn vertex(
    @location(0) pos : vec4<f32>,   // vec4 – matches 32-B vertex stride
    @location(1) nrm : vec4<f32>
) -> VSOut {
    var out : VSOut;
    out.world_pos = pos.xyz;
    out.pos       = view.view_proj * vec4<f32>(pos.xyz, 1.0);
    return out;
}

@fragment
fn fragment(in : VSOut) -> @location(0) vec4<f32> {
    // simple normal-based colour
    return vec4<f32>((in.world_pos * 0.5) + 0.5, 1.0);
}
