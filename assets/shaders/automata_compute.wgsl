//--------------------------------------------------------------------
// generic_automata_compute.wgsl   (single entry‑point)
//
// • Discrete rules (Conway, Wolfram) are handled by a 512‑entry LUT
//   passed as a read‑only storage buffer.
// • Float rules (Lenia) pass a convolution kernel + param block.
// • The shader chooses the right branch via `rule_kind` in AutomataGlobals.
//--------------------------------------------------------------------

struct AutomataGlobals {
    board_size : vec2<u32>,
    rule_kind  : u32,     // 0 = LUT‑9, 1 = conv‑float, …
    padding    : u32,
};
@group(0) @binding(2) var<uniform> G : AutomataGlobals;

@group(0) @binding(0)
var<storage, read>        state_in  : texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1)
var<storage, read_write>  state_out : texture_storage_2d<rgba8unorm, write>;

// Kind‑0 rule data ----------------------------------------------------
@group(0) @binding(3)
var<storage, read> rule_table : array<u32>;   // 512 × u32 : next state

// Kind‑1 rule data (Lenia) -------------------------------------------
@group(0) @binding(4)
var<storage, read> conv_kernel : texture_storage_2d<rgba32float, read>;
@group(0) @binding(5)
var<uniform>       lenia_params : vec4<f32>;  // mu, sigma, dt, _

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) id : vec3<u32>) {
    if (any(id.xy >= G.board_size)) { return; }

    // read current state ------------------------------------------------
    let self_px = textureLoad(state_in, vec2<i32>(id.xy), 0).r;

    if (G.rule_kind == 0u) {
        // ----------------------------------------------------------------
        // Conway / LUT‑9
        // ----------------------------------------------------------------
        var idx : u32 = 0u;
        var bit : u32 = 0u;
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let n = textureLoad(
                            state_in,
                            vec2<i32>(vec2<i32>(id.xy) + vec2<i32>(dx, dy)),
                            0).r > 0.5;
                idx |= select(0u, 1u << bit, n);
                bit += 1u;
            }
        }
        let next_state = f32(rule_table[idx] & 1u);
        textureStore(state_out, vec2<i32>(id.xy), vec4<f32>(next_state, 0.0, 0.0, 1.0));
    } else {
        // ----------------------------------------------------------------
        // Lenia float path (separable convolution omitted for brevity)
        // ----------------------------------------------------------------
        // accumulate kernel × state   …
    }
}
