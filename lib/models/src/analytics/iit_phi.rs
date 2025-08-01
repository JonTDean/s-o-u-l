//! Integrated Information Theory (IIT) proxy metric Φ for a voxel world.
//!
//! Φ ≈ 1  → one large, highly integrated cluster  
//! Φ → 0  → many tiny, disconnected clusters

use engine_core::world::voxel_world::VoxelWorld;
use super::clustering::find_clusters;   // ← bring the helper into scope

/// Compute the Φ proxy for the current world state.
pub fn compute_phi(world: &VoxelWorld) -> f32 {
    let clusters = find_clusters(world);
    let total_alive: usize = clusters.iter().map(|c| c.size).sum();
    if total_alive < 2 { return 0.0; }

    let largest = clusters.iter().map(|c| c.size).max().unwrap_or(0);
    largest as f32 / total_alive as f32
}
