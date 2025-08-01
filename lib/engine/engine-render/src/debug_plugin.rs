//! Debug logging – no CPU grid, so we just dump slice info.

use bevy::prelude::*;
use engine_core::prelude::*;

use crate::WorldCamera;


pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (dump_registry, dump_render_map.after(dump_registry), dump_camera.after(dump_render_map)),
        );
    }
}

fn dump_registry(reg: Res<AutomataRegistry>) {
    for a in reg.list() {
        info!(target: "soul::auto",
              id = ?a.id,
              name = %a.name,
              size = ?a.slice.size,
              voxel = %a.voxel_size,
              "registry (slice-only)");
    }
}

fn dump_render_map() {}


fn dump_camera(cam_q: Query<(&Transform, &Projection), With<WorldCamera>>) {
    if let Ok((tf, Projection::Orthographic(o))) = cam_q.single() {
        info!(target: "soul::cam",
              pos = ?tf.translation.truncate(),
              scale = %o.scale,
              "world-camera");
    }
}
