//! Integrated Information Theory (IIT) – compute an approximate Φ (phi) metric for a grid world.
//! We use a simplistic proxy: we find connected clusters of "alive" cells and compute an 
//! integration index as the fraction of live cells in the largest cluster. A highly integrated 
//! state (one big cluster) yields Φ ≈ 1, while many separate small clusters yield a lower Φ.

use engine_core::world::World2D;

use crate::analytics::swarm_metrics::find_clusters;

/// Computes a simple integrated information metric Φ for the given world state.
pub fn compute_phi(world: &World2D) -> f32 {
    // Use cluster analysis as a proxy for integrated information
    let clusters = find_clusters(world);
    let total_alive: usize = clusters.iter().map(|c| c.size).sum();
    if total_alive < 2 {
        return 0.0; // No integration possible with fewer than 2 live cells
    }
    // Find largest cluster
    let largest = clusters.iter().max_by_key(|c| c.size).map(|c| c.size).unwrap_or(0);
    // Φ as fraction of cells in largest integrated cluster
    largest as f32 / total_alive as f32
}
