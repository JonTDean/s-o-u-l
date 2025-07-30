//! Information Theory of Individuality (ITI) – identify distinct individuals (clusters) 
//! and measure their autonomy (internal vs external interactions).

use engine::core::world::World2D;
use crate::analytics::swarm_metrics::{find_clusters, ClusterStats};

/// Identifies clusters of alive cells and returns their autonomy metrics.
/// Each cluster's autonomy is the fraction of its neighbor interactions that are internal 
/// (within the cluster) as opposed to external (with other clusters).
pub fn identify_individuals(world: &World2D) -> Vec<ClusterStats> {
    find_clusters(world)
    // `find_clusters` already computes internal/external link counts and autonomy for each cluster.
}
