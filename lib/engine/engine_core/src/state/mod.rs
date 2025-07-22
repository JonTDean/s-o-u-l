//! State crate-wide “hub”.
//!
//! • Re-exports the public `AppState` enum so call-sites can simply write  
//!   `crate::state::AppState` instead of digging into sub-paths.
//! • Re-exports the `StatePlugin` so your `main.rs` can stay concise.

pub mod app_state;   // ✅ your enum lives here
pub mod plugin;      // ✅ the Bevy plugin we added
pub mod resources;   // ✅ long-lived Resources

pub use app_state::AppState;
pub use plugin::StatePlugin;
