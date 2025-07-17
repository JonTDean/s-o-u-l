//! Namespace for everything that belongs to the **main-menu MVC stack**.

pub mod view;        // View
pub mod model;      // Pure data
pub mod controller; // Controllers implementing `MenuScreen`

// ─── Re-exports so call-sites keep the old identifiers ──────────────────────
pub use controller::{LoadScenario, NewScenario, OptionsScreen};
pub use view::MainMenu;
