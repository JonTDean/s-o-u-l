//! 3-D grid back-ends (dense & sparse).

pub mod dense;
pub mod sparse;

use serde::{Deserialize, Serialize};

pub use dense ::DenseGrid;
pub use sparse::SparseGrid;

/// Run-time grid selection.
#[derive(Clone, Serialize, Deserialize)]
pub enum GridBackend {
    Dense (DenseGrid),
    Sparse(SparseGrid),
}
