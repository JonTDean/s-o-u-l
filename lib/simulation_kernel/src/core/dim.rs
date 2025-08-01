//! Spatial dimensionality – **3-D only**.
//!
//! `Dim` is a unit struct that represents left-handed voxel space
//! (`+X → right`, `+Y → up`, `+Z → forward`).  It implements the
//! [`Dimensionality`] trait, exposing the compile-time Moore-26 offsets.

use glam::IVec3;

/// Compile-time abstraction over dimensionality.
///
/// Keeping this trait lets us preserve the generic kernels while the
/// engine is single-dimensional (3-D).  Adding new dimensions later will
/// only require *implementing* the trait, not rewriting callers.
pub trait Dimensionality: Copy + Eq + std::hash::Hash + Send + Sync + 'static {
    type Coord: Copy + Eq + std::hash::Hash + Send + Sync;
    const NEIGHBOUR_OFFSETS: &'static [Self::Coord];
}

/// **Voxel space** unit struct.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Dim;

impl Dimensionality for Dim {
    type Coord = IVec3;

    /// Compile-time Moore-26 neighbourhood (pre-computed).
    const NEIGHBOUR_OFFSETS: &'static [Self::Coord] = {
        const fn build() -> [IVec3; 26] {
            let mut arr = [IVec3::ZERO; 26];
            let mut i = 0;
            let mut z = -1;
            while z <= 1 {
                let mut y = -1;
                while y <= 1 {
                    let mut x = -1;
                    while x <= 1 {
                        if !(x == 0 && y == 0 && z == 0) {
                            arr[i] = IVec3::new(x, y, z);
                            i += 1;
                        }
                        x += 1;
                    }
                    y += 1;
                }
                z += 1;
            }
            arr
        }
        &build()
    };
}
