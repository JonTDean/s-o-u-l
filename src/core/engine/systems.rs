use bevy::prelude::*;

/// Placeholder world-stepping system.
///
/// The real version will read the grid + rule resources and update cell
/// components.  For now it only prints so we can verify schedule order.
pub fn step_world() {
    trace!(">> Logic stage – world stepped");
}
