//! engine-gpu / compute / **module hub**
//!
//! This directory contains **all GPU-only extraction stages** that sit
//! _between_ the simulation field (voxel atlas) and the final raster
//! pass.  Each stage is exposed as a dedicated *render-graph node* so
//! higher-level code can reorder, disable or multi-queue them without
//! tight coupling.
//!
//! ## Layout
//! ```text
//! compute/
//! ├── dual_contour.rs   ← guaranteed path (all GPUs)
//! ├── mesh_path.rs      ← optional VK_EXT_mesh_shader fast-path
//! └── mod.rs            ← you-are-here
//! ```
//!
//! ### Thread-Safety & Bevy Sub-Apps
//! * All heavy work runs on the GPU; the CPU side only builds command
//!   buffers, therefore:
//!   * **Send + Sync** everywhere.
//!   * No main-thread stalls – the render sub-app owns the nodes.
//! * Each module exposes its *public* node type (`DualContourNode`,
//!   `MeshPathNode`) so the compositor can register them in any graph.

pub mod dual_contour;
#[cfg(feature = "mesh_shaders")]
pub mod mesh_path;