//! Explicit per-automaton control state shared with the render world.
use bevy::prelude::*;
use simulation_kernel::core::lenia::{LeniaParams, LeniaSeed};
mod commands;
mod snapshot;
pub use snapshot::SavedLenia;
#[cfg(test)]
mod tests;
pub use commands::{LeniaAction, LeniaCommand, LeniaControlPlugin};

/// Desired simulation state; tick requests survive slow rendering and extraction.
#[derive(Component, Clone, Debug, Default)]
pub struct LeniaSimulation {
    /// Optional completed checkpoint used when allocating GPU buffers.
    pub restored: Option<std::sync::Arc<SavedLenia>>,
    /// Validated growth controls.
    pub params: LeniaParams,
    /// Deterministic reset fixture.
    pub seed: LeniaSeed,
    /// Whether automatic fixed-clock requests are suspended.
    pub paused: bool,
    /// Total requested ticks since the latest reset.
    pub target_tick: u64,
    /// Reset generation; changing it discards all old GPU state.
    pub epoch: u64,
}

impl LeniaSimulation {
    /// Restore the seed on the next render extraction while retaining controls.
    pub fn reset(&mut self) {
        self.restored = None;
        self.epoch += 1;
        self.target_tick = 0;
    }
}

/// True while project save owns the shared `LeniaProbe` mailbox; metric sampling waits.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadbackBusy(pub bool);
