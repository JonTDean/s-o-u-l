// Minimal Lenia step – just copies input to output so the rule runs.
@group(0) @binding(0)
var atlas : texture_3d<u32>;                // read-only, use `textureLoad`

@compute @workgroup_size(16,16,1)
fn main(@builtin(global_invocation_id) id : vec3<u32>) {
    // no‑op placeholder
}
