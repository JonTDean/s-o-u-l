// life_binary.wgsl - Compute shader for Conway's Game of Life (binary states)
// Now supports batched layers (Z dimension), ping-pong buffering, and signal field output.

struct AutomatonParams {
    size_x:  u32;
    size_y:  u32;
    layer:   u32;
    rule:    u32;
    offset_x: u32;
    offset_y: u32;
};

@group(0) @binding(0) var<storage, read_write> ping_tex   : texture_storage_3d<r8uint, read_write>;
@group(0) @binding(1) var<storage, read_write> pong_tex   : texture_storage_3d<r8uint, read_write>;
@group(0) @binding(2) var<uniform>            frame_parity: u32;
@group(0) @binding(3) var<storage, read>      params_arr  : array<AutomatonParams>;
@group(0) @binding(4) var<storage, read_write> signal_tex : texture_storage_3d<r8uint, read_write>;

const OFFS: array<vec2<i32>, 8> = array<vec2<i32>, 8>(
    vec2(-1, -1), vec2(0, -1), vec2(1, -1),
    vec2(-1,  0),               vec2(1,  0),
    vec2(-1,  1), vec2(0,  1), vec2(1,  1),
);

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let layer_index = gid.z;
    // Ensure we don't index beyond the array (in case of over-dispatch).
    if (layer_index >= arrayLength(&params_arr)) {
        return;
    }
    let params = params_arr[layer_index];
    // Discard threads outside the bounds of this automaton's grid.
    if (gid.x >= params.size_x || gid.y >= params.size_y) {
        return;
    }

    // Compute the absolute coordinate in the 3D texture for this cell.
    let world_x = i32(params.offset_x + gid.x);
    let world_y = i32(params.offset_y + gid.y);
    let layer   = i32(params.layer);
    let coord   = vec3<i32>(world_x, world_y, layer);

    // Load current state from the appropriate texture (ping or pong) based on frame parity.
    var self_state: u32;
    var live_neighbors: u32 = 0u;
    if (frame_parity == 0u) {
        self_state = textureLoad(ping_tex, coord, 0).r;
        // Count live neighbors from ping texture
        for (var i = 0; i < 8; i++) {
            let n = coord.xy + OFFS[i];
            live_neighbors += textureLoad(ping_tex, vec3<n, coord.z>, 0).r;
        }
    } else {
        self_state = textureLoad(pong_tex, coord, 0).r;
        for (var i = 0; i < 8; i++) {
            let n = coord.xy + OFFS[i];
            live_neighbors += textureLoad(pong_tex, vec3<n, coord.z>, 0).r;
        }
    }

    // Apply Conway's Life rules (using the 32-bit packed rule mask).
    let born = ((params.rule >> live_neighbors) & 1u) == 1u && self_state == 0u;
    let stay = ((params.rule >> (live_neighbors + 9u)) & 1u) == 1u && self_state == 1u;
    let new_state = if (born || stay) { 1u } else { 0u };

    // Write the new state to the opposite texture (ping<->pong swap).
    if (frame_parity == 0u) {
        textureStore(pong_tex, coord, vec4<u32>(new_state, 0u, 0u, 0u));
    } else {
        textureStore(ping_tex, coord, vec4<u32>(new_state, 0u, 0u, 0u));
    }

    // Also output to the signal field (e.g., mark where live cells are).
    textureStore(signal_tex, coord, vec4<u32>(new_state, 0u, 0u, 0u));
}
