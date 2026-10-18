//! Topology, atlas offsets, transforms, empty scenes and bounded overflow.
use super::gpu::run;

#[test]
fn one_voxel_has_six_outward_faces_at_the_requested_world_transform() {
    let (vertices, draw) = run(&[255], [1, 1, 1], 36, [3, 5, 2]);
    assert_eq!(draw, [36, 1, 0, 0]);
    assert_eq!(vertices.len(), 36);
    for vertex in &vertices {
        assert!((10.0..=14.0).contains(&vertex[0]));
        assert!((20.0..=24.0).contains(&vertex[1]));
        assert!((30.0..=34.0).contains(&vertex[2]));
        assert_eq!(vertex[3], 1.0);
        assert_eq!(vertex[4].abs() + vertex[5].abs() + vertex[6].abs(), 1.0);
    }
    for triangle in vertices.chunks_exact(3) {
        let a = bevy::math::Vec3::from_slice(&triangle[0][..3]);
        let b = bevy::math::Vec3::from_slice(&triangle[1][..3]);
        let c = bevy::math::Vec3::from_slice(&triangle[2][..3]);
        let normal = bevy::math::Vec3::from_slice(&triangle[0][4..7]);
        assert!((b - a).cross(c - a).dot(normal) > 0.0);
    }
}

#[test]
fn adjacent_voxels_cull_their_shared_face() {
    let (_, draw) = run(&[255, 255], [2, 1, 1], 72, [0, 0, 0]);
    assert_eq!(draw, [60, 1, 0, 0]);
}

#[test]
fn empty_region_emits_no_vertices() {
    let (vertices, draw) = run(&[0; 8], [2, 2, 2], 36, [2, 2, 2]);
    assert_eq!(draw, [0, 1, 0, 0]);
    assert!(vertices.is_empty());
}

#[test]
fn overflow_clamps_indirect_draw_to_whole_faces() {
    let (vertices, draw) = run(&[255; 8], [2, 2, 2], 13, [0, 0, 0]);
    assert_eq!(draw, [12, 1, 0, 0]);
    assert_eq!(vertices.len(), 12);
}

#[test]
fn preview_volume_matches_a_cpu_exposed_face_oracle() {
    let cells = crate::seed::demo::fixture();
    let mut faces = 0;
    for z in 0i32..16 {
        for y in 0i32..16 {
            for x in 0i32..16 {
                if cells[(z * 256 + y * 16 + x) as usize] == 0 {
                    continue;
                }
                for (dx, dy, dz) in [
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, 1, 0),
                    (0, -1, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                ] {
                    let (nx, ny, nz) = (x + dx, y + dy, z + dz);
                    if !(0..16).contains(&nx)
                        || !(0..16).contains(&ny)
                        || !(0..16).contains(&nz)
                        || cells[(nz * 256 + ny * 16 + nx) as usize] == 0
                    {
                        faces += 1;
                    }
                }
            }
        }
    }
    let (_, draw) = run(&cells, [16, 16, 16], 36 * 4096, [32, 32, 8]);
    assert_eq!(draw, [faces * 6, 1, 0, 0]);
    assert!(draw[0] > 0);
}
