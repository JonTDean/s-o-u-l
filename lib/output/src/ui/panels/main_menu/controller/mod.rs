//! Controller namespace for the **main-menu MVC stack**.
//!
//! Each concrete screen lives in its own sub-module (e.g. `scenario/new.rs`).
//! This `mod.rs` file wires those sub-modules together and re-exports the
//! public screen types so call-sites can keep using the original identifiers
//! such as `NewScenario`, `LoadScenario`, and `OptionsScreen`.

#![allow(clippy::module_inception)] // We deliberately use `mod.rs` as a namespace module

// ─── Sub-modules ─────────────────────────────────────────────────────────────
pub mod scenario;
pub mod options;

// Re-export screen types for convenience
pub use scenario::new::NewScenario;
pub use scenario::load::LoadScenario;
pub use options::OptionsScreen;
