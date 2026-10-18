//! Explicit GPU readback requests; ordinary rendering performs no readbacks.
use bevy::prelude::*;
use simulation_kernel::core::lenia::LeniaParams;
mod readback;
pub(crate) use readback::{PendingReadbacks, complete, record};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

/// State actually read back from the GPU after a submitted Lenia update.
#[derive(Clone, Debug)]
pub struct LeniaSnapshot {
    /// Reset generation captured with the command buffer.
    pub epoch: u64,
    /// Completed simulation ticks represented by the density buffer.
    pub tick: u64,
    /// Local volume dimensions, independent of atlas origin.
    pub size: [u32; 3],
    /// Parameters used by the latest dispatch.
    pub params: LeniaParams,
    /// Quantized density values in X-fastest order.
    pub densities: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct ProbeState {
    pub active: usize,
    pub requested: HashSet<Entity>,
    pub results: HashMap<Entity, Result<LeniaSnapshot, String>>,
}

/// Shared main/render-world mailbox for deliberate runtime verification.
#[derive(Resource, Clone, Default)]
pub struct LeniaProbe(pub(crate) Arc<Mutex<ProbeState>>);

impl LeniaProbe {
    /// Number of live GPU state slots after the latest render synchronization.
    pub fn active_instances(&self) -> usize {
        self.0.lock().expect("Lenia probe lock").active
    }

    /// Request one asynchronous readback of this instance on its next draw.
    pub fn request(&self, owner: Entity) {
        self.0
            .lock()
            .expect("Lenia probe lock")
            .requested
            .insert(owner);
    }

    /// Consume a finished readback, including any GPU mapping failure.
    pub fn take(&self, owner: Entity) -> Option<Result<LeniaSnapshot, String>> {
        self.0
            .lock()
            .expect("Lenia probe lock")
            .results
            .remove(&owner)
    }
}
