//! Information-theoretic individuality: identify autonomous clusters
//! and expose their statistics (size, internal/external links, autonomy).

use engine_core::world::voxel_world::VoxelWorld;

use super::clustering::{find_clusters, ClusterStats};

/// Returns one `ClusterStats` entry per individual (cluster).
pub fn identify_individuals(world: &VoxelWorld) -> Vec<ClusterStats> {
    find_clusters(world)
}
