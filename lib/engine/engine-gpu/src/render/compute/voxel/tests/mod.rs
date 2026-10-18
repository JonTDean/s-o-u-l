//! Shader schema checks and real GPU surface/readback regression tests.
mod gpu;
mod surfaces;

#[test]
fn active_shaders_validate() {
    for source in [
        super::pipelines::SURFACE_SHADER,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/voxel_mesh.wgsl"
        )),
    ] {
        let module = naga::front::wgsl::parse_str(source).expect("WGSL parses");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("WGSL validates");
    }
    use bevy::render::render_resource::ShaderType;
    assert_eq!(crate::AutomatonParams::min_size().get(), 20);
    assert_eq!(std::mem::size_of::<crate::AutomatonParams>(), 20);
    assert_eq!(std::mem::size_of::<super::buffers::Vertex>(), 32);
    assert_eq!(std::mem::size_of::<super::SurfaceParams>(), 48);
}
