use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub fn spawn_debug_floor(
    mut commands : Commands,
    mut meshes   : ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // floor -----------------------------------------------------------
    let floor_mesh = meshes.add(
        Plane3d { normal: Dir3::Z, half_size: Vec2::splat(1_000.0) }
    );
    let floor_mat  = materials.add(Color::srgb(0.15,0.15,0.15));

    commands.spawn((
        Mesh3d::from(floor_mesh),
        MeshMaterial3d::from(floor_mat),
    ));

    // shared axis cylinder -------------------------------------------
    let axis_mesh = meshes.add(Cylinder { radius: 0.05, half_height: 250.0 });

    for (rot, colour) in [
        (Quat::from_rotation_y(FRAC_PI_2), Color::srgb(1.0,0.0,0.0)),
        (Quat::IDENTITY,                Color::srgb(0.0,1.0,0.0)),
        (Quat::from_rotation_x(FRAC_PI_2), Color::srgb(0.0,0.0,1.0)),
    ] {
        commands.spawn((
            Mesh3d::from(axis_mesh.clone()),
            MeshMaterial3d::from(materials.add(colour)),
            Transform::from_rotation(rot),
        ));
    }
}
