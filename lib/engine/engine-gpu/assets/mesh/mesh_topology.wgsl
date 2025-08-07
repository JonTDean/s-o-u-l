// WGSL mesh **mesh** shader – outputs a single triangle.
//
// Mesh shader outputs must declare the primitive type & maximum verts.
enable chromium_experimental_full_rendering;

struct VSOut { @builtin(position) pos : vec4<f32>; };

@mesh @output(topology = "triangle-list", max_vertices = 3, max_primitives = 1)
fn main() -> (vertices : array<VSOut, 3>, primitives : array<u32, 1>) {
    vertices[0].pos = vec4<f32>(-0.5, -0.5, 0.0, 1.0);
    vertices[1].pos = vec4<f32>( 0.5, -0.5, 0.0, 1.0);
    vertices[2].pos = vec4<f32>( 0.0,  0.5, 0.0, 1.0);
    primitives[0] = 0u;
}
