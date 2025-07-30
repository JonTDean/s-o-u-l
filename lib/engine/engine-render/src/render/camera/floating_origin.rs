//! Floating-origin maintenance.
//!
//! This system keeps the active [`WorldCamera`] centred around the
//! global origin to avoid floating-point precision loss in very large
//! worlds. When the camera wanders further than one “chunk” (`SNAP`)
//!  from (0, 0), we:
//!
//! 1. **Re-centre** the camera by an integer number of chunks.
//! 2. **Shift every other entity** the opposite way so the player
//!    experiences no visual discontinuity.
//! 3. **Accumulate** the displacement in the [`WorldOffset`] resource so
//!    render materials, GPU shaders, etc. can stay in world-space.
//!
//! ## Concurrency
//!
//! Bevy requires that *mutable* queries accessing the same component
//! (`Transform` here) are provably disjoint.  Although the filters
//! `With<WorldCamera>` and `(Without<Camera>, Without<WorldCamera>)` are
//! logically exclusive, the borrow-checker cannot establish this at
//! compile-time.  Wrapping the two queries in a [`ParamSet`] gives them
//! independent lifetimes and satisfies the ECS aliasing rules without
//! impacting run-time scheduling or parallelism.
//!
//! The system is therefore completely safe to run in parallel with
//! anything else that only **reads** `Transform`, and Bevy may schedule
//! it on any available thread.

use bevy::prelude::*;

use crate::render::camera::systems::WorldCamera;

/* --------------------------------------------------------------------- */
/*                               Constants                               */
/* --------------------------------------------------------------------- */

/// Size of one snap “chunk” in world units  
/// (e.g. 256 cells if `cell_size == 1.0`).
const SNAP: i32 = 256;

/* --------------------------------------------------------------------- */
/*                                Resources                              */
/* --------------------------------------------------------------------- */

/// Global world-offset (in **world units**, already multiplied by
/// `cell_size`).  Render materials & GPU shaders query this every frame.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct WorldOffset(pub IVec2);

/* --------------------------------------------------------------------- */
/*                                 System                                */
/* --------------------------------------------------------------------- */

/// Re-centres the camera when it drifts beyond ±`SNAP` in either axis.
///
/// ### Scheduling
/// * Should run **after** any camera-movement system (pan/zoom/drag) so
///   it sees the final camera transform for the current frame.
/// * Can run in parallel with systems that *read* `Transform`.  
///   (Bevy handles the actual thread assignment.)
///
/// ### Parameters
/// * `ParamSet` – disjoint mutable access to `Transform`:
///   * `p0` → the **single** [`WorldCamera`]
///   * `p1` → **every other** entity that does *not* have `Camera` or
///            `WorldCamera` (most of the world)
/// * `WorldOffset` – mutable resource accumulating the total shift.
pub fn apply_floating_origin(
    mut q: ParamSet<(
        Query<&mut Transform, With<WorldCamera>>,
        Query<&mut Transform, (Without<Camera>, Without<WorldCamera>)>,
    )>,
    mut offset: ResMut<WorldOffset>,
) {
    /* ── 0. Early-out when no world camera is active ─────────────────── */
    let mut binding = q.p0();
    let mut cam_tf = match binding.single_mut() {
        Ok(t) => t,
        Err(_) => return, // WorldCamera not spawned yet
    };

    /* ── 1. Has the camera strayed beyond the snap distance? ─────────── */
    if cam_tf.translation.x.abs() < SNAP as f32
        && cam_tf.translation.y.abs() < SNAP as f32
    {
        return; // Still close to origin – nothing to do.
    }

    /* ── 2. Compute whole-chunk displacement (integer, signed) ───────── */
    let dx = (cam_tf.translation.x / SNAP as f32).round() as i32;
    let dy = (cam_tf.translation.y / SNAP as f32).round() as i32;

    let delta = Vec3::new(
        -(dx as f32) * SNAP as f32,
        -(dy as f32) * SNAP as f32,
        0.0,
    );

    /* ── 3. Re-centre the camera ─────────────────────────────────────── */
    cam_tf.translation += delta;

    /* ── 4. Shift every non-camera entity in the opposite direction ──── */
    for mut tf in q.p1().iter_mut() {
        tf.translation -= delta;
    }

    /* ── 5. Record the accumulated offset for render logic ───────────── */
    offset.0 += IVec2::new(dx * SNAP, dy * SNAP);
}
