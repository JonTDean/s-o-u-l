// The first field matches Bevy ViewUniform::clip_from_world. The explicit
// bind layout retains Bevy's full minimum size and dynamic view offset.
struct View { clip_from_world: mat4x4<f32> };
@group(0) @binding(0) var<uniform> view: View;
struct VSOut {
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) density: f32,
    @builtin(position) pos: vec4<f32>,
};
@vertex
fn vertex(@location(0) pos: vec4<f32>, @location(1) normal: vec4<f32>) -> VSOut {
    var out: VSOut;
    out.pos = view.clip_from_world * pos;
    out.normal = normal.xyz;
    out.world_pos = pos.xyz;
    out.density = normal.w;
    return out;
}
struct PaletteEntry { rect: vec4<f32>, stops: array<vec4<f32>, 4> };
struct Palette { header: vec4<u32>, entries: array<PaletteEntry, 8> };
@group(0) @binding(1) var<uniform> palette: Palette;

// Amber ramp (linear light) used where no channel rectangle contains the fragment.
fn default_stop(i: u32) -> vec3<f32> {
    switch i {
        case 0u: { return vec3<f32>(0.0176, 0.0144, 0.0097); }
        case 1u: { return vec3<f32>(0.2423, 0.1274, 0.0331); }
        case 2u: { return vec3<f32>(1.0000, 0.4621, 0.0630); }
        default: { return vec3<f32>(1.0000, 0.6867, 0.3663); }
    }
}

fn ramp_at(world: vec2<f32>, density: f32) -> vec3<f32> {
    let t = clamp(density, 0.0, 1.0) * 3.0;
    let i = min(u32(floor(t)), 2u);
    let f = t - f32(i);
    for (var k = 0u; k < palette.header.x; k = k + 1u) {
        let r = palette.entries[k].rect;
        if (world.x >= r.x && world.x <= r.z && world.y >= r.y && world.y <= r.w) {
            return mix(palette.entries[k].stops[i].xyz, palette.entries[k].stops[i + 1u].xyz, f);
        }
    }
    return mix(default_stop(i), default_stop(i + 1u), f);
}

@fragment
fn fragment(in: VSOut) -> @location(0) vec4<f32> {
    let light = 0.35 + 0.65 * max(dot(normalize(in.normal), normalize(vec3<f32>(0.4, 0.8, 0.6))), 0.0);
    let band = 0.8 + 0.2 * sin(in.world_pos.y * 0.12);
    // Lift low surface densities so the ramp's colour stop dominates the silhouette.
    let shade = ramp_at(in.world_pos.xy, 0.35 + 0.65 * in.density);
    return vec4<f32>(shade * light * band, 1.0);
}
