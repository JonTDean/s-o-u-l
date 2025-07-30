//! Swarm statistics and cluster detection for agent-based systems on the grid.
//! Provides metrics like cluster count, average cluster size, etc., and identifies cluster structure.

use bevy::math::IVec2;
use engine::core::world::World2D;
use simulation_kernel::{core::{cell::CellState, dim::{Dim, Dim2}}, grid::GridBackend};

#[derive(Debug)]
pub struct ClusterStats {
    pub id: usize,
    pub size: usize,
    pub internal_links: usize,
    pub external_links: usize,
    pub autonomy: f32,
}

/// Finds all clusters of adjacent alive cells in the world and returns their statistics.
pub fn find_clusters(world: &World2D) -> Vec<ClusterStats> {
    let mut clusters: Vec<ClusterStats> = Vec::new();
    let mut cluster_id_map_dense: Option<Vec<isize>> = None;
    let mut cluster_id_map_sparse: Option<std::collections::HashMap<IVec2, isize>> = None;
    let mut current_id: isize = 0;
    // Determine grid type
    match &world.backend {
        GridBackend::Dense(grid) => {
            let width = grid.size.x as i32;
            let height = grid.size.y as i32;
            let mut cluster_id = vec![-1isize; (width * height) as usize];
            // Helper to get index from coord
            let idx = |p: IVec2| -> usize { (p.y as u32 * grid.size.x + p.x as u32) as usize };
            // DFS to label clusters
            for (i, cell) in grid.cells.iter().enumerate() {
                if !matches!(cell.state, CellState::Dead) && cluster_id[i] == -1 {
                    // New cluster found
                    let cid = current_id;
                    current_id += 1;
                    let mut stack = vec![IVec2::new((i as u32 % grid.size.x) as i32, (i as u32 / grid.size.x) as i32)];
                    cluster_id[i] = cid;
                    let mut count = 0;
                    // Depth-first fill
                    while let Some(pos) = stack.pop() {
                        count += 1;
                        // Explore neighbors
                        for off in Dim2::NEIGHBOUR_OFFSETS.iter() {
                            let npos = pos + *off;
                            if npos.x < 0 || npos.x >= width || npos.y < 0 || npos.y >= height {
                                continue;
                            }
                            let ni = idx(npos);
                            if cluster_id[ni] == -1 {
                                if let Some(neigh) = grid.cells.get(ni) {
                                    if !matches!(neigh.state, CellState::Dead) {
                                        cluster_id[ni] = cid;
                                        stack.push(npos);
                                    }
                                }
                            }
                        }
                    }
                    clusters.push(ClusterStats {
                        id: cid as usize,
                        size: count,
                        internal_links: 0,
                        external_links: 0,
                        autonomy: 0.0,
                    });
                }
            }
            cluster_id_map_dense = Some(cluster_id);
        }
        GridBackend::Sparse(grid) => {
            let mut cluster_map: std::collections::HashMap<IVec2, isize> = std::collections::HashMap::new();
            let mut unvisited: std::collections::HashSet<IVec2> = grid.map.iter().filter_map(|(coord, cell)| {
                if !matches!(cell.state, CellState::Dead) { Some(*coord) } else { None }
            }).collect();
            while let Some(&start) = unvisited.iter().next() {
                // New cluster
                let cid = current_id;
                current_id += 1;
                let mut stack = vec![start];
                cluster_map.insert(start, cid);
                unvisited.remove(&start);
                let mut count = 0;
                while let Some(pos) = stack.pop() {
                    count += 1;
                    for off in Dim2::NEIGHBOUR_OFFSETS.iter() {
                        let npos = pos + *off;
                        if unvisited.contains(&npos) {
                            // neighbor is alive and not yet visited
                            cluster_map.insert(npos, cid);
                            unvisited.remove(&npos);
                            stack.push(npos);
                        }
                    }
                }
                clusters.push(ClusterStats {
                    id: cid as usize,
                    size: count,
                    internal_links: 0,
                    external_links: 0,
                    autonomy: 0.0,
                });
            }
            cluster_id_map_sparse = Some(cluster_map);
        }
    }
    // Second pass: compute internal/external neighbor links for each cluster
    for cluster in clusters.iter_mut() {
        let cid = cluster.id as isize;
        let mut internal_sum = 0;
        let mut external_sum = 0;
        match &world.backend {
            GridBackend::Dense(grid) => {
                let width = grid.size.x as i32;
                let height = grid.size.y as i32;
                let cluster_id = cluster_id_map_dense.as_ref().unwrap();
                for (i, cell) in grid.cells.iter().enumerate() {
                    if cluster_id[i] == cid && !matches!(cell.state, CellState::Dead) {
                        // For each neighbor
                        let x = (i as u32 % grid.size.x) as i32;
                        let y = (i as u32 / grid.size.x) as i32;
                        let pos = IVec2::new(x, y);
                        for off in Dim2::NEIGHBOUR_OFFSETS.iter() {
                            let npos = pos + *off;
                            if npos.x < 0 || npos.x >= width || npos.y < 0 || npos.y >= height {
                                continue;
                            }
                            let ni = (npos.y as u32 * grid.size.x + npos.x as u32) as usize;
                            if let Some(neigh) = grid.cells.get(ni) {
                                if !matches!(neigh.state, CellState::Dead) {
                                    if cluster_id[ni] == cid {
                                        internal_sum += 1;
                                    } else {
                                        external_sum += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            GridBackend::Sparse(grid) => {
                let cluster_map = cluster_id_map_sparse.as_ref().unwrap();
                for (&pos, &c) in cluster_map.iter() {
                    if c == cid {
                        // For each neighbor position
                        for off in Dim2::NEIGHBOUR_OFFSETS.iter() {
                            let npos = pos + *off;
                            if let Some(&nc) = cluster_map.get(&npos) {
                                if nc == cid {
                                    internal_sum += 1;
                                } else {
                                    external_sum += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        // Each adjacency between two cells in the same cluster was counted twice (once from each side)
        cluster.internal_links = internal_sum / 2;
        cluster.external_links = external_sum;
        let total_links = (cluster.internal_links * 2) + cluster.external_links;
        // If no links at all (isolated single cell), treat autonomy as 1 (fully independent)
        cluster.autonomy = if total_links == 0 {
            1.0
        } else {
            (cluster.internal_links * 2) as f32 / total_links as f32
        };
    }
    clusters
}

/// Computes high-level swarm statistics for the current world.
pub fn swarm_summary(world: &World2D) -> SwarmStats {
    let clusters = find_clusters(world);
    let total_agents: usize = clusters.iter().map(|c| c.size).sum();
    let cluster_count = clusters.len();
    let largest_cluster = clusters.iter().max_by_key(|c| c.size).map(|c| c.size).unwrap_or(0);
    let average_cluster = if cluster_count > 0 {
        total_agents as f32 / cluster_count as f32
    } else {
        0.0
    };
    let singletons = clusters.iter().filter(|c| c.size == 1).count();
    SwarmStats {
        total_agents,
        cluster_count,
        largest_cluster,
        average_cluster,
        singletons,
    }
}

/// Summary metrics for a swarm-like system.
pub struct SwarmStats {
    pub total_agents: usize,
    pub cluster_count: usize,
    pub largest_cluster: usize,
    pub average_cluster: f32,
    pub singletons: usize,
}
