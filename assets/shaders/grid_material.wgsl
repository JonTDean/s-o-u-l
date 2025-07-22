// ─────────────────────────────────────────────────────────────────────────────
// GridMaterial shader  –  Bevy 0.16-compatible
//
// Streams the current automata state into a scrolling **RGBA8** texture used
// by the debug / overview renderer.  Much like the AutomataMaterial above,
// the key bug was the missing model matrix, leading to an unscaled 1×1 quad.
//
// The fix mirrors the one in *automata_material.wgsl*.
// ─────────────────────────────────────────────────────────────────────────────

#import bevy_sprite::mesh2d_functions::{
    get_world_from_local,
    mesh2d_position_local_to_world,
    mesh2d_position_world_to_clip,
}

// ─────– Group 2 – grid parameters + texture ─────────────────────────────────
struct GridParams {
    /// Top-left texel *of the visible camera window* in world units.
    /// (Updated every frame by Rust code.)
    origin   : vec2<f32>,
    /// Texture size in texels.
    tex_size : vec2<f32>,
};

@group(2) @binding(0) var<uniform> G          : GridParams;
@group(2) @binding(1) var          state_tex  : texture_2d<f32>;
@group(2) @binding(2) var          state_samp : sampler;

// ─────– Per-vertex I/O ───────────────────────────────────────────────────────
struct VertexInput {
    @location(0) position        : vec3<f32>,
    @builtin(instance_index) iid : u32,
};

struct VOut {
    @builtin(position) clip : vec4<f32>,
    @location(0)      uv   : vec2<f32>,
};

// ─────– Vertex stage ─────────────────────────────────────────────────────────
@vertex
fn vertex(in: VertexInput) -> VOut {
    var o : VOut;

    // Apply full model transform (+ per-instance)
    let w_from_l = get_world_from_local(in.iid);
    let world    = mesh2d_position_local_to_world(w_from_l, vec4<f32>(in.position, 1.0));

    // World → clip for the GPU
    o.clip = mesh2d_position_world_to_clip(world);

    // World → UV (wrap-around handled in Rust by scrolling `G.origin`)
    o.uv   = (world.xy + G.origin) / G.tex_size;

    return o;
}

// ─────– Fragment stage ───────────────────────────────────────────────────────
@fragment
fn fragment(in: VOut) -> @location(0) vec4<f32> {
    let v = textureSample(state_tex, state_samp, in.uv).r;
    return vec4<f32>(v, v, v, 1.0);
}
