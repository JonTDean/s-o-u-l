//! Published organism checkpoint used by library and native qualification.
use engine_core::automata::lenia::SavedLenia;
use simulation_kernel::core::lenia::{
    LeniaSeed,
    continuous::{State, orbium},
};
pub fn specimen() -> SavedLenia {
    let size = [64, 64, 8];
    let continuous = State {
        kernel: orbium::KERNEL,
        values: orbium::seed(size),
    };
    SavedLenia {
        size,
        tick: 0,
        params: orbium::PARAMS,
        seed: LeniaSeed::Orbium,
        paused: true,
        densities: continuous.bytes(),
        continuous: Some(continuous),
    }
}
