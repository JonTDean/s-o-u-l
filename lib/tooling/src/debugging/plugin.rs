use bevy::prelude::*;
use engine_core::prelude::*;


/// Seconds between two consecutive log lines.
const LOG_INTERVAL: f64 = 30.0;

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

/* ──────────────────────────────────────────────────────────────── */
/* Registry logging – throttled                                     */
/* ──────────────────────────────────────────────────────────────── */
fn dump_registry(
    time:  Res<Time>,
    mut last: Local<f64>,                     // per-system timestamp
    reg:   Res<AutomataRegistry>,
) {
    if time.elapsed_secs_f64() - *last < LOG_INTERVAL {
        return;
    }
    *last = time.elapsed_secs_f64();

    for a in reg.list() {
        info!(target: "soul::auto", id=?a.id, name=%a.name, "registry entry");
    }
}

/* ──────────────────────────────────────────────────────────────── */
/* Active-camera logging – throttled                                */
/* ──────────────────────────────────────────────────────────────── */
fn dump_camera(
    time:  Res<Time>,
    mut last: Local<f64>,                     // per-system timestamp
    cam_q: Query<(&Transform, &Projection, &Camera)>,
) {
    if time.elapsed_secs_f64() - *last < LOG_INTERVAL {
        return;
    }
    *last = time.elapsed_secs_f64();

    for (tf, proj, cam) in &cam_q {
        if !cam.is_active {
            continue;
        }
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
