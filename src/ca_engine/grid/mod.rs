pub mod dense;
pub mod sparse;

pub use dense::DenseGrid;
pub use sparse::SparseGrid;

pub enum GridBackend {
    Dense(DenseGrid),
    Sparse(SparseGrid),
}