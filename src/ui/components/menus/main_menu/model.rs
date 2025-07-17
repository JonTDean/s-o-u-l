//! Data-only “model” layer for the main-menu option screens.
//!
//! These structs contain **no UI code** and no Bevy‐specific lifetimes;
//! they are therefore easy to test in isolation and can be shared between
//! game logic and editor tooling.

/// Draft parameters the user edits when creating a new scenario.
///
/// *Thread-safety*: `u32` is `Send + Sync`, so the struct is as well.
#[derive(Debug, Default)]
pub struct ScenarioDraft {
    pub width:  u32,
    pub height: u32,
}

/// Placeholder for the load-scenario workflow (file path, preview meta…).
#[derive(Debug, Default)]
pub struct LoadScenarioData;

/// Persistent user settings from the **Options** screen.
#[derive(Debug, Default)]
pub struct UiSettings {
    pub font_size: f32,
}
