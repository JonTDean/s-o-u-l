//! Fixed‑step simulation accumulator & events.
//!
//! This module provides two core resources – [`FixedStepConfig`] and
//! [`SimAccumulator`] – plus the [`SimulationStep`] event and the
//! [`accumulate_and_step`] system.  Together they implement the classic
//! *“fix your timestep”* game‑loop inside Bevy’s variable‑rate `Update`
//! schedule.
//!
//! ### How it works
//! * Each video frame, [`accumulate_and_step`] adds the real‑time
//!   `delta_seconds()` to an accumulator.
//! * While the accumulator ≥ `dt` (and we haven’t hit the per‑frame cap),
//!   we **emit** a [`SimulationStep`] event and subtract `dt`.
//! * Systems that perform deterministic simulation (physics, automata,
//!   AI, etc.) **listen** for `SimulationStep` and advance exactly one
//!   tick per event.
//! * If a frame runs long, multiple events are emitted; if the frame is
//!   faster than `dt`, **zero** events are sent – rendering merely
//!   interpolates.
//!
//! The design mirrors Unity’s *FixedUpdate* vs *Update* separation, but we
//! keep everything in the main world for maximum flexibility.

use std::time::Duration;
use bevy::prelude::*;

/* ===================================================================== */
/* Resources                                                             */
/* ===================================================================== */

/// Immutable run‑time parameters for the fixed‑step loop.
#[derive(Resource, Debug, Clone, Copy)]
pub struct FixedStepConfig {
    /// Length of one simulation tick.
    pub dt: Duration,
    /// Maximum allowed steps per **render frame** (spiral‑of‑death guard).
    pub max_steps_per_frame: u8,
}
impl FixedStepConfig {
    #[inline]
    pub fn from_hz(hz: u32, max_steps_per_frame: u8) -> Self {
        let dt_secs = 1.0 / hz as f64;
        Self { dt: Duration::from_secs_f64(dt_secs), max_steps_per_frame }
    }
}

/// Accumulates real‑time between simulation ticks.
#[derive(Resource, Default, Debug)]
pub struct SimAccumulator {
    pub accum: f64,   // seconds
}

/* ===================================================================== */
/* Event                                                                 */
/* ===================================================================== */

/// Emitted **once per simulation tick**.
#[derive(Event, Debug, Clone, Copy)]
pub struct SimulationStep;

/* ===================================================================== */
/* System                                                                */
/* ===================================================================== */

/// Turns frame‑delta into 0‑N [`SimulationStep`] events.
///
/// Add this system to any `Update` stage **before** simulation logic runs.
/// It is completely CPU‑only and thread‑safe.
#[allow(clippy::needless_pass_by_value)]
pub fn accumulate_and_step(
    time: Res<Time>,
    cfg:  Res<FixedStepConfig>,
    mut acc: ResMut<SimAccumulator>,
    mut step_writer: EventWriter<SimulationStep>,
) {
    let dt_sec    = cfg.dt.as_secs_f64();
    acc.accum    += time.delta_secs_f64();

    let mut steps = 0u8;
    while acc.accum >= dt_sec && steps < cfg.max_steps_per_frame {
        step_writer.write(SimulationStep);
        acc.accum -= dt_sec;
        steps     += 1;
    }

    // Drop any excess to avoid spiral‑of‑death (optional design choice).
    if steps == cfg.max_steps_per_frame && acc.accum >= dt_sec {
        // WARNING: simulation running too slow – clamp.
        info!(target: "soul::sim", "Accumulator clamped: dropped {:.3} ms of lag", acc.accum * 1_000.0);
        acc.accum = 0.0;
    }
}
