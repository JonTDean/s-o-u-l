//! A static volume and oblique camera exercise geometry, transforms, and depth.
use bevy::prelude::*;
use engine_common::controls::camera::{WorldCamera, manager::layers_world};
use engine_core::{automata::GpuGridSlice, prelude::AppState};

pub(super) fn setup(mut commands: Commands, mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::InGame);
    commands.spawn(GpuGridSlice {
        layer: 8,
        offset: UVec2::new(32, 32),
        size: UVec2::splat(16),
        depth: 16,
        rule: "voxel-demo".into(),
        rule_bits: 0,
    });
    camera(
        &mut commands,
        Vec3::new(160.0, 154.0, 32.0),
        "S.O.U.L. / Static voxel preview",
    );
    info!("Voxel preview: 16³ fixture at atlas (32,32,8), 4 world units per voxel");
}

pub(super) fn camera(commands: &mut Commands, centre: Vec3, label: &str) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Camera {
            order: 3,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.025, 0.035, 0.055)),
            ..default()
        },
        Transform::from_translation(centre + Vec3::new(90.0, 65.0, 120.0))
            .looking_at(centre, Vec3::Y),
        layers_world(),
    ));
    commands.spawn((
        Text::new(label),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.78, 0.85, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            bottom: Val::Px(24.0),
            ..default()
        },
    ));
}

pub(super) fn isolate_preview_camera(mut cameras: Query<&mut Camera, With<WorldCamera>>) {
    for mut camera in &mut cameras {
        camera.is_active = false;
    }
}
