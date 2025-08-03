// ───────────────────────────────────────────────────────────────
// Debug floor shader — draws a static XY grid with optional alpha
// opacity. Purely visual, not tied to voxel atlas or simulation.
//
// Vertex: full quad
// Fragment: renders lines every `GRID_STEP` in world-space.
// ───────────────────────────────────────────────────────────────

const GRID_STEP: f32 = 8.0;
const LINE_THICKNESS: f32 = 0.04;

@group(2) @binding(0) var<uniform> Params : DebugFloorParams;

struct DebugFloorParams {
    zoom:      f32,
    alpha:     f32,
    _padding:  vec2<f32>,
    origin:    vec2<f32>,  // in world-space
};

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       world_pos:     vec2<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VOut {
    var out: VOut;
    let pos_world = in.position.xy + Params.origin;
    out.world_pos = pos_world;
    out.clip_position = vec4<f32>(in.position.xy, 0.0, 1.0);
    return out;
}

@fragment
fn fragment(in: VOut) -> @location(0) vec4<f32> {
    let x_mod = abs(fract(in.world_pos.x / GRID_STEP) - 0.5);
    let y_mod = abs(fract(in.world_pos.y / GRID_STEP) - 0.5);
    let x_line = step(0.5 - LINE_THICKNESS, x_mod);
    let y_line = step(0.5 - LINE_THICKNESS, y_mod);
    let line = 1.0 - min(x_line, y_line);
    return vec4<f32>(0.8, 0.8, 0.8, line * Params.alpha);
}
