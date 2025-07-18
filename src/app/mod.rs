//! High‑level application orchestration.
//!
//! The **`app`** module exposes a single public entry‑point
//! (`builder::run`) that handles **all** runtime configuration:
//!   * Parsing CLI / config‑file arguments  
//!   * Spinning up a tuned, multi‑threaded Bevy [`App`] instance  
//!   * Registering *every* core, feature and utility plugin in the
//!     precise order defined in the architecture docs.
//!
//! Down‑stream crates should *never* construct their own `App`; instead,
//! they call `app::builder::run()` from their `main()`, ensuring one
//! consistent startup path for the entire executable.

pub mod builder;
pub mod runner;
pub mod schedule;
pub mod plugin_registry;