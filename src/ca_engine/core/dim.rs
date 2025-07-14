use bevy::math::IVec2;

/// Trait implemented by every supported spatial dimensionality.
///
/// Keeping the offsets in a `const` slice allows the compiler to fully
/// unroll the tight loops in the stepper.
pub trait Dim: Copy + Eq + std::hash::Hash + Send + Sync + 'static {
    type Coord: Copy + Eq + std::hash::Hash + Send + Sync;
    const NEIGHBOUR_OFFSETS: &'static [Self::Coord];
}

/// Two‑dimensional grid using the Moore‑8 neighbourhood.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Dim2;

impl Dim for Dim2 {
    type Coord = IVec2;

    const NEIGHBOUR_OFFSETS: &'static [Self::Coord] = &[
        IVec2::new(-1, -1), IVec2::new(0, -1), IVec2::new(1, -1),
        IVec2::new(-1, 0),                     IVec2::new(1, 0),
        IVec2::new(-1, 1),  IVec2::new(0, 1),  IVec2::new(1, 1),
    ];
}