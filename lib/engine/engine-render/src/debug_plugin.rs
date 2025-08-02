//! render/debug_plugin.rs – minimal logging-only variant.
//!
//! You can re-enable them later with a proper 3-D gizmo implementation.

use bevy::prelude::*;
use engine_core::prelude::*;
use crate::WorldCamera;

#[derive(Default)]
pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dump_registry,
                dump_camera.after(dump_registry),
            ),
        );
    }
}

/* ───── tiny diagnostics ───── */
fn dump_registry(reg: Res<AutomataRegistry>) {
    for a in reg.list() {
        info!(target: "soul::auto", id=?a.id, name=%a.name, "registry entry");
    }
}
fn dump_camera(cam_q: Query<(&Transform, &Projection), With<WorldCamera>>) {
    if let Ok((tf, proj)) = cam_q.single() {
        match proj {
            Projection::Orthographic(o) =>
                info!(target: "soul::cam", pos=?tf.translation.truncate(), scale=%o.scale),
            Projection::Perspective(_) =>
                info!(target: "soul::cam", pos=?tf.translation.truncate(), "(perspective)"),
            Projection::Custom(_) =>
                info!(target: "soul::cam", pos=?tf.translation.truncate(), "(custom)"),
        }
    }
}
