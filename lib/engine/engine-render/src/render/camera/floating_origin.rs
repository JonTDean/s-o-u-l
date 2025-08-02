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
use tooling::debugging::camera::CameraDebug;

/* --------------------------------------------------------------------- */
/*                                Resources                              */
/* --------------------------------------------------------------------- */

/// Global world-offset (in **world units**, already multiplied by
/// `cell_size`).  Render materials & GPU shaders query this every frame.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct WorldOffset(pub IVec2);

/* ───────────────────── Helpers ─────────────────────── */

/// Next power-of-two of the *current* half-viewport so snaps never occur
/// on-screen.
///
/// *Patched*: use `Window::width/height` (logical) instead of
/// `physical_width/height` so behaviour is consistent on Hi-DPI displays.
#[inline]
fn dynamic_snap(win: &Window, scale: f32) -> f32 {
    let half = (win.width().max(win.height()) * 0.5 * scale) as u32;
    half.next_power_of_two() as f32
}

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
    windows: Query<&Window>,
    mut q: ParamSet<(
        Query<&mut Transform, With<WorldCamera>>,                       // 0 – camera
        Query<&mut Transform, (
            Without<Camera>, 
            Without<WorldCamera>, 
            Without<tooling::debugging::floor::NoOriginShift>
        )>, // 1 – every other entity
    )>,
    mut offset: ResMut<WorldOffset>,
    debug: Res<CameraDebug>,
) {
    /* 0 fetch window + camera safely -------------------------------- */
    let win                                 = match  windows.single()          { Ok(w) => w,  Err(_) => return };
    let mut cam_q0 = q.p0();                          // <-- bind first
    let mut cam_tf               = match  cam_q0.single_mut()       { Ok(t) => t, Err(_) => return };

    /* 1 early-out when still in the safe zone ----------------------- */
    let snap = dynamic_snap(win, 1.0);                                      // proj-scale already baked into translation
    if cam_tf.translation.x.abs() < snap && cam_tf.translation.y.abs() < snap {
        return;
    }

    /* 2 integer displacement in snap units -------------------------- */
    let dx = (cam_tf.translation.x / snap).round() as i32;
    let dy = (cam_tf.translation.y / snap).round() as i32;
    let delta = Vec3::new(-(dx as f32) * snap, -(dy as f32) * snap, 0.0);

    /* 3 shift camera ------------------------------------------------ */
    cam_tf.translation += delta;

    /* 4 shift the rest of the world --------------------------------- */
    for mut tf in q.p1().iter_mut() {
        tf.translation -= delta;
    }

    /* 5 accumulate world offset ------------------------------------- */
    offset.0 += IVec2::new((snap as i32) * dx, (snap as i32) * dy);

    /* 6 optional debug logging -------------------------------------- */
    if debug.contains(CameraDebug::LOG_SNAP) {
        info!("Floating-origin snap Δ=({dx},{dy})  →  offset = {:?}", offset.0);
    }
}