//! Spawns the single *world* camera once a simulation world exists.
//!
//! The camera is shared by **all** world‑space renderers (active mask,
//! debug grid, gizmos, …).  By putting the logic in its own plugin we
//! avoid duplication and – most importantly – ensure the camera is only
//! created *after* the `World2D` resource has been initialised.

use bevy::{
    core_pipeline::core_2d::Camera2d,
    ecs::schedule::common_conditions::resource_exists,
    prelude::*,
};
use engine_core::{core::World2D, engine::grid::GridBackend, state::AppState};
use input::controls::camera_control::{WorldCamera, ZoomInfo};

/// Marker resource – `true` after the camera has been spawned.
#[derive(Resource, Default)]
pub struct WorldCameraSpawned(pub bool);

/// System: spawn the camera exactly **once** per app run.
fn spawn_world_camera_once(
    mut cmd:  Commands,
    mut flag: ResMut<WorldCameraSpawned>,
    world:    Res<World2D>,
) {
    if flag.0 {
        return; // already spawned
    }

    // Decide a sensible initial focus point ------------------------------
    let (w, h) = match &world.backend {
        GridBackend::Dense(g)  => (g.size.x, g.size.y),
        GridBackend::Sparse(_) => (512, 512),
    };

    cmd.spawn((
        Camera2d,
        // All world quads share this priority;
        // UI cameras use much larger values (e.g. 100).
        Camera { order: 2, ..default() },
        Transform::from_translation(Vec3::new(
            w as f32 * world.cell_size * 0.5,
            h as f32 * world.cell_size * 0.5,
            999.0,                         // far in front
        )),
        WorldCamera,
    ));
    cmd.insert_resource(ZoomInfo { base: 1.0, current: 1.0 });

    flag.0 = true;
}

/// Tiny plugin that installs the above system.
///
/// * runs only while `AppState::InGame` is active;
/// * waits until `World2D` exists – prevents the panic seen before.
pub struct WorldCameraPlugin;
impl Plugin for WorldCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCameraSpawned>()
            .add_systems(
                Update,
                spawn_world_camera_once
                    .run_if(in_state(AppState::InGame))
                    .run_if(resource_exists::<World2D>),
            );
    }
}
