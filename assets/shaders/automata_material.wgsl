// ─────────────────────────────────────────────────────────────────────────────
// AutomataMaterial shader  –  Bevy 0.16-compatible
//
// Renders an **R8-unorm** texture that encodes cell life state
// (0 = dead, 1 = alive) on a world-aligned quad.
//
// Key fix in this version
// -----------------------
// The original shader ignored Bevy’s per-instance **model matrix** and sent the
// *local* vertex position straight to clip-space.  That broke every transform
// (translation, rotation, scale) applied from Rust – hence the “mysterious”
// giant black square that never matched the simulation grid.
//
// We now:
//
//   1. Import Bevy’s *mesh2d* helper functions and bindings.
//   2. Convert **local → world** and **world → clip** with the proper matrix.
//   3. Pass the true world-space position to the fragment stage.
//
// With those changes the quad scales to the logical grid size, the camera pans
// & zooms correctly, and texture sampling lines up 1-to-1 with automata cells.
// ─────────────────────────────────────────────────────────────────────────────

// ─────– Bevy helper imports (handle transform + view matrices) ───────────────
#import bevy_sprite::mesh2d_functions::{
    get_world_from_local,
    mesh2d_position_local_to_world,
    mesh2d_position_world_to_clip,
}

// ─────– Group 2 – material data + grid texture ───────────────────────────────
struct AutomataParams {
    camera_pos:   vec2<f32>,   // world-space camera centre
    zoom:         f32,         // orthographic zoom factor
    cell_size:    f32,         // one cell in world units
    texture_size: vec2<f32>,   // (width, height) in texels
    dead_color:   vec4<f32>,   // RGBA for state = 0
    alive_color:  vec4<f32>,   // RGBA for state = 1
};

@group(2) @binding(0) var<uniform> Params    : AutomataParams;
@group(2) @binding(1) var          grid_tex  : texture_2d<f32>;
@group(2) @binding(2) var          grid_samp : sampler;

// ─────– Per-vertex I/O ───────────────────────────────────────────────────────
struct VertexInput {
    @location(0) position        : vec3<f32>,
    @builtin(instance_index) iid : u32,
};

struct VOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0)      world_pos      : vec2<f32>,
};

// ─────– Vertex stage ─────────────────────────────────────────────────────────
@vertex
fn vertex(in: VertexInput) -> VOut {
    var out : VOut;

    // full **world_from_local** matrix for this instance
    let w_from_l = get_world_from_local(in.iid);

    // Convert local → world → clip in the canonical Bevy way
    let world = mesh2d_position_local_to_world(w_from_l, vec4<f32>(in.position, 1.0));
    out.clip_position = mesh2d_position_world_to_clip(world);
    out.world_pos     = world.xy;               // hand over for UV math

    return out;
}

// ─────– Fragment stage ───────────────────────────────────────────────────────
@fragment
fn fragment(in: VOut) -> @location(0) vec4<f32> {
    // 1. Translate into camera (“view”) space
    let world_offset = in.world_pos - Params.camera_pos;

    // 2. Convert world units → cell units (respecting zoom)
    let cell_scale   = Params.cell_size * Params.zoom;
    let tex_space    = world_offset / cell_scale;

    // 3. Map to texture coordinates (0 … 1), treating the grid centre as (0, 0)
    let tex_uv = (tex_space + Params.texture_size * 0.5) / Params.texture_size;

    // 4. Sample R8 texture (nearest-neighbour set in Rust)
    let state = textureSample(grid_tex, grid_samp, tex_uv).r;

    // 5. Blend dead ↔ alive colours
    return mix(Params.dead_color, Params.alive_color, state);
}
