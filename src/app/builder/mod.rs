//! Builder subsystem root.
//!
//! This module bundles configuration parsing and, eventually, higher‑level
//! helpers for constructing a fully‑tuned Bevy [`App`].  Splitting the
//! original *monolithic* `builder.rs` into smaller, purpose‑focused units
//! improves testability and allows parallel work on configuration loading,
//! asset pre‑flighting, and plugin orchestration.
//!
//! ## Directory layout
//! ````text
//! builder/
//! ├── mod.rs          ← *you are here* – public façade & re‑exports
//! └── config.rs       ← RuntimeConfig definition & loaders
//! ````
//!
//! Additional helper files (e.g. `assets.rs`, `pipeline.rs`) may be added
//! later without changing imports elsewhere, because callers only depend on
//! the façade.
//!
//! ## Public re‑exports
//! * [`RuntimeConfig`] – fully‑resolved runtime parameters
//! * [`load_runtime_config()`] – shorthand around `RuntimeConfig::load()`
//!
//! ```rust
//! use app::builder::RuntimeConfig;
//! let cfg = app::builder::load_runtime_config();
//! ```

pub mod config;

pub use config::RuntimeConfig;

/// Convenience wrapper around [`RuntimeConfig::load`].
#[inline]
#[allow(dead_code)] 
pub fn load_runtime_config() -> RuntimeConfig {
    RuntimeConfig::load()
}
