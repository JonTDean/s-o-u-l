pub mod dense;
pub mod sparse;

pub use dense::DenseGrid;
pub use sparse::SparseGrid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum GridBackend {
    Dense(DenseGrid),
    Sparse(SparseGrid),
}