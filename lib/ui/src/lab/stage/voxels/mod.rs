//! Dedicated stage camera and ECS voxel presentation for simple symbolic automata.
pub mod layout;
use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use engine_common::controls::camera::manager::stage::{StageRect, physical_viewport};
use engine_core::automata::channel::Channels;
use models::workbench::{plugin::Workbench, session::Model, symbolic::machine::Symbolic};
#[derive(Component)]
pub struct SymbolVoxel;
#[derive(Component)]
pub struct VoxelCamera;
#[derive(Resource)]
pub struct Meshes {
    cube: Handle<Mesh>,
    materials: std::collections::HashMap<char, Handle<StandardMaterial>>,
    head: Handle<StandardMaterial>,
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let head = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });
    commands.insert_resource(Meshes {
        cube: meshes.add(Cuboid::new(2.3, 2.3, 2.3)),
        materials: default(),
        head,
    });
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 4,
            is_active: false,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.065, 0.063, 0.058)),
            ..default()
        },
        Msaa::Off,
        VoxelCamera,
        RenderLayers::layer(20),
        Transform::from_xyz(0.0, 25.0, 95.0).looking_at(Vec3::new(0.0, -8.0, 0.0), Vec3::Y),
    ));
}
pub fn is_symbolic(model: &Model) -> bool {
    matches!(model, Model::Symbolic(m) if !matches!(m, Symbolic::Koch { .. }))
}
#[allow(clippy::too_many_arguments)]
pub fn sync(
    mut commands: Commands,
    bench: Res<Workbench>,
    channels: Res<Channels>,
    rect: Res<StageRect>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<VoxelCamera>>,
    entities: Query<Entity, With<SymbolVoxel>>,
    mut assets: ResMut<Meshes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut previous: Local<Vec<layout::Cell>>,
) {
    let model = bench
        .sessions
        .iter()
        .find(|s| Some(s.channel) == channels.selected)
        .map(|s| &s.model);
    let cells = match model {
        Some(Model::Symbolic(m)) => layout::cells(m),
        _ => Vec::new(),
    };
    for mut camera in &mut cameras {
        camera.viewport = windows
            .single()
            .ok()
            .and_then(|w| physical_viewport(&rect, w.physical_size()));
        camera.is_active = !cells.is_empty() && rect.managed && camera.viewport.is_some();
    }
    if *previous == cells {
        return;
    }
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    for cell in &cells {
        let material = assets
            .materials
            .entry(cell.symbol)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: layout::palette(cell.symbol).0,
                    perceptual_roughness: 0.75,
                    ..default()
                })
            })
            .clone();
        commands.spawn((
            SymbolVoxel,
            Mesh3d(assets.cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(cell.position),
            RenderLayers::layer(20),
        ));
        if cell.head {
            commands.spawn((
                SymbolVoxel,
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(assets.head.clone()),
                Transform::from_translation(cell.position + Vec3::Y * 3.0)
                    .with_scale(Vec3::new(1.0, 0.2, 1.0)),
                RenderLayers::layer(20),
            ));
        }
    }
    *previous = cells;
}
