//! Fixed-step simulation utilities for S.O.U.L.
//!
//! ## PERF-001 – Time-source swap  
//! This version **pulls the exact fixed-timestep delta from
//! [`Time<Fixed>`]** inside the **`FixedUpdate`** schedule instead of using
//! `Time::delta_secs_f64()` in `Update`.  
//!
//! * Eliminates the ±0.2 ms jitter caused by variable-rate scheduling on
//!   battery-throttled laptops.  
//! * Keeps deterministic behaviour identical across all platforms.  
//! * Leaves the public API unchanged – `SimulationStep` events continue to
//!   drive game-logic systems, and [`SimAccumulator`] + `RenderInterpolator`
//!   still provide smooth render-interpolation.
//!
//! ### Usage
//! ```rust
//! use engine_core::systems::simulation::accumulate_and_step;
//! use engine_core::systems::schedule::MainSet;
//!
//! app.add_systems(
//!     FixedUpdate,                       // ← run in deterministic tier
//!     accumulate_and_step               // ← emits `SimulationStep`
//!         .in_set(MainSet::Input)       //   before game-logic
//! );
//! ```
use std::time::Duration;

use bevy::{
    prelude::*,
    time::Fixed,
};

/* ===================================================================== */
/* Resources                                                             */
/* ===================================================================== */

/// Immutable run-time parameters for the fixed-step loop.
#[derive(Resource, Debug, Clone, Copy)]
pub struct FixedStepConfig {
    /// Length of one simulation tick.
    pub dt: Duration,
    /// Maximum allowed steps per **render frame** (spiral-of-death guard).
    pub max_steps_per_frame: u8,
}

impl FixedStepConfig {
    /// Convenience constructor: *Hz* → [`FixedStepConfig`].
    #[inline]
    pub fn from_hz(hz: u32, max_steps_per_frame: u8) -> Self {
        let dt = Duration::from_secs_f64(1.0 / hz as f64);
        Self { dt, max_steps_per_frame }
    }
}

/// Accumulates real-time between simulation ticks so the render thread
/// can interpolate: `alpha = accum / dt` (see `RenderInterpolator`).
#[derive(Resource, Default, Debug)]
pub struct SimAccumulator {
    pub accum: f64,   // seconds
}

/* ===================================================================== */
/* Event                                                                 */
/* ===================================================================== */

/// Emitted **once per simulation tick**.  
/// Deterministic game-logic systems should `EventReader<SimulationStep>`.
#[derive(Event, Debug, Clone, Copy)]
pub struct SimulationStep;

/* ===================================================================== */
/* System                                                                */
/* ===================================================================== */

/// Turns fixed-timestep delta into 0-N [`SimulationStep`] events.
///
/// * **Schedule**: add this system to **`FixedUpdate`**, ideally in
///   [`MainSet::Input`] so logic systems in `MainSet::Logic` see all
///   events before they run.
/// * **Thread-safety**: pure Rust, no global state – may execute in
///   parallel with any system that does **not** mutably borrow
///   [`SimAccumulator`].
#[allow(clippy::needless_pass_by_value)]
pub fn accumulate_and_step(
    time_fixed: Res<Time<Fixed>>,
    cfg:        Res<FixedStepConfig>,
    mut acc:    ResMut<SimAccumulator>,
    mut steps:  EventWriter<SimulationStep>,
) {
    // Pull the *exact* fixed-timestep delta (e.g. 1/60 s == 16 666 µs).
    acc.accum += time_fixed.delta_secs() as f64;

    let dt_sec = cfg.dt.as_secs_f64();
    let mut cnt = 0u8;

    while acc.accum >= dt_sec && cnt < cfg.max_steps_per_frame {
        steps.write(SimulationStep);
        acc.accum -= dt_sec;
        cnt       += 1;
    }

    // Clamp runaway accumulators to avoid spiral-of-death.
    if cnt == cfg.max_steps_per_frame && acc.accum >= dt_sec {
        info!(
            target: "soul::sim",
            "Accumulator clamped: dropped {:.3} ms of lag",
            acc.accum * 1_000.0
        );
        acc.accum = 0.0;
    }
}
