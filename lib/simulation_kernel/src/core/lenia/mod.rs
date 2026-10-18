//! Quantized XY-plane Lenia semantics shared by CPU rules and GPU validation.
mod params;
mod reference;
pub use params::LeniaParams;
pub use reference::{KERNEL, next_density, step};

#[cfg(test)]
mod tests;
