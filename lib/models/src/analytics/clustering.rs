//! Generic 3-D cluster detection + statistics utilities.
//!
//! A *cluster* is a set of orthogonally / diagonally adjacent live voxels
//! (26-neighbour connectivity).

use glam::IVec3;
use std::collections::{HashSet, VecDeque};

use engine_core::world::voxel_world::VoxelWorld;
use simulation_kernel::{
    core::dim::{Dim, Dimensionality},                    // for NEIGHBOUR_OFFSETS
};

use super::swarm_metrics::live_cell_set;

/* ─────────────────────────── public data type ─────────────────────────── */

#[derive(Clone, Debug)]
pub struct ClusterStats {
    /// Number of voxels in the cluster.
    pub size: usize,
    /// Undirected links whose both ends lie *inside* the cluster.
    pub internal_links: usize,
    /// Undirected links with exactly one end in the cluster.
    pub external_links: usize,
}

impl ClusterStats {
    /// Autonomy: fraction of neighbour interactions that remain internal.
    pub fn autonomy(&self) -> f32 {
        let tot = self.internal_links + self.external_links;
        if tot == 0 { 0.0 } else { self.internal_links as f32 / tot as f32 }
    }
}

/* ─────────────────────────── main entry point ─────────────────────────── */

/// Breadth-first search over the live-cell graph (26-connectivity).
pub fn find_clusters(world: &VoxelWorld) -> Vec<ClusterStats> {
    let live: HashSet<IVec3> = live_cell_set(&world.backend);
    let mut visited: HashSet<IVec3> = HashSet::new();
    let mut out = Vec::new();

    for &seed in &live {
        if visited.contains(&seed) { continue; }

        // BFS to gather one cluster
        let mut q = VecDeque::new();
        q.push_back(seed);
        visited.insert(seed);

        let mut members = Vec::new();
        while let Some(p) = q.pop_front() {
            members.push(p);
            for off in Dim::NEIGHBOUR_OFFSETS {
                let n = p + *off;
                if live.contains(&n) && visited.insert(n) {
                    q.push_back(n);
                }
            }
        }

        // ── compute link statistics ─────────────────────────────────────
        let mut internal = 0usize;
        let mut external = 0usize;
        let member_set: HashSet<_> = members.iter().copied().collect();

        for &p in &members {
            for off in Dim::NEIGHBOUR_OFFSETS {
                let n = p + *off;
                if live.contains(&n) {
                    if member_set.contains(&n) {
                        internal += 1;
                    } else {
                        external += 1;
                    }
                }
            }
        }
        // each internal edge counted twice (A→B, B→A)
        internal /= 2;

        out.push(ClusterStats {
            size: members.len(),
            internal_links: internal,
            external_links: external,
        });
    }
    out
}
