//! debug_grid.rs — persistent, zoom-aware checker grid drawn with a
//! Material2d.  Updated for Bevy 0.16.1 and the new required-components API.
//
//! Author: Jon (Obaven Inc.) — 2025-08-03
//! ---------------------------------------------------------------------------

use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use engine_common::controls::camera::WorldCamera;

use crate::render::materials::debug::debug_floor::{DebugFloorMaterial, DebugFloorParams};

/* ─────────────────────────────────────────────────────────────── Components */

/// Tag so we can query the grid entity later.
#[derive(Component)]
pub(crate) struct DebugGridTag;

/// Stores the material handle once the grid is spawned, avoiding the
/// “`Handle<T>` is not a Component” compile error.
#[derive(Resource, Deref, DerefMut)]
struct DebugGridMatHandle(Handle<DebugFloorMaterial>);

/* ────────────────────────────────────────────────────────────────  Plugin */

/// Adds a zoom-stable debug grid to every 2-D scene.
pub struct DebugGridPlugin;

impl Plugin for DebugGridPlugin {
    fn build(&self, app: &mut App) {
        app
            // register custom material
            .add_plugins(Material2dPlugin::<DebugFloorMaterial>::default())
            // spawn once
            .add_systems(Startup, spawn_debug_grid)
            // update each frame
            .add_systems(Update, update_debug_grid);
    }
}

/* ──────────────────────────────────────────────────────────────── Systems */

/// Create a single, huge quad in clip-space so the grid always covers the view.
fn spawn_debug_grid(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DebugFloorMaterial>>,
) {
    // 2× default camera size ⇒ fills view even at extreme zoom-out.
    const GRID_EXTENTS: f32 = 2_048.0;

    // Procedural mesh from the new Rectangle primitive.
    let mesh_handle = meshes.add(Rectangle::new(GRID_EXTENTS, GRID_EXTENTS));

    // Material with sensible defaults; `DebugFloorParams` now derives Default.
    let mat_handle = materials.add(DebugFloorMaterial {
        params: DebugFloorParams {
            zoom:  1.0,
            alpha: 0.2,
            ..Default::default()
        },
    });

    // Spawn the renderable entity using the modern component pair.
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(mat_handle.clone()),
        Transform::from_xyz(0.0, 0.0, -1.0), // render behind everything else
        DebugGridTag,
    ));

    // Cache the material handle for quick access in `update_debug_grid`.
    commands.insert_resource(DebugGridMatHandle(mat_handle));
}

/// Drive grid parameters from the camera every frame so lines stay crisp.
fn update_debug_grid(
    camera:  Query<(&Transform, &Projection), With<WorldCamera>>,
    mat_res: Res<DebugGridMatHandle>,
    mut mats: ResMut<Assets<DebugFloorMaterial>>,
) {
    // Using `single()` (returns Result) per 0.15+ API. Ignore errors silently.
    let Ok((cam_tf, proj)) = camera.single() else { return };

    if let Some(mat) = mats.get_mut(&**mat_res) {
        // Sync zoom with orthographic scale.
        if let Projection::Orthographic(ortho) = proj {
            mat.params.zoom = ortho.scale;
        }
        // Keep the grid origin pinned to camera translation.
        mat.params.origin = cam_tf.translation.truncate();
    }
}
