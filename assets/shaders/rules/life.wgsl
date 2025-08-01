struct Params {
    size_x  : u32;
    size_y  : u32;
    layer   : u32;
    rule    : u32;   // not used for Life
    offset_x: u32;
    offset_y: u32;
};

@group(0) @binding(0) var<storage, read>  src : texture_3d<u32>;
@group(0) @binding(1) var<storage, read_write> dst : texture_3d<u32>;
@group(0) @binding(2) var<uniform> parity : u32;
@group(0) @binding(3) var<storage, read> params : array<Params>;

@compute @workgroup_size(16,16,1)
fn main(@builtin(global_invocation_id) gid : vec3<u32>,
        @builtin(workgroup_id) wid      : vec3<u32>) {
    let slice = params[wid.z];
    if (gid.x >= slice.size_x || gid.y >= slice.size_y) { return; }

    let x = slice.offset_x + gid.x;
    let y = slice.offset_y + gid.y;
    let z = slice.layer;

    // Moore-neighbourhood sum
    var live : u32 = 0u;
    for (var dy : i32 = -1; dy <= 1; dy++) {
        for (var dx : i32 = -1; dx <= 1; dx++) {
            if (dx == 0 && dy == 0) { continue; }
            live += textureLoad(src, vec3<i32>(i32(x)+dx,i32(y)+dy,i32(z)), 0).r;
        }
    }
    let self = textureLoad(src, vec3<i32>(x,y,z), 0).r;
    let next = select(
        select(0u, 1u, live == 3u),   // dead -> born on 3
        select(1u, 0u, live < 2u || live > 3u), // live -> stay / die
        self == 1u);

    textureStore(dst, vec3<i32>(x,y,z), vec4<u32>(next,0u,0u,0u));
}
