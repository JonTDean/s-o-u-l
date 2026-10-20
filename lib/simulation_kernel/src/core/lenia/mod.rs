//! Quantized XY-plane Lenia semantics shared by CPU rules and GPU validation.
mod params;
mod reference;
mod seed;
pub use params::LeniaParams;
pub use reference::{KERNEL, next_density, step};
pub use seed::{LeniaSeed, seed};

pub mod continuous;
#[cfg(test)]
mod tests;
