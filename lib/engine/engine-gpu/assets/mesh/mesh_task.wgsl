// WGSL mesh **task** shader – generates one task (workgroup) per draw.
enable chromium_experimental_full_rendering;

@task @workgroup_size(1)
fn main() -> u32 {           // returns number of mesh workgroups
    return 1u;               // one mesh workgroup
}
