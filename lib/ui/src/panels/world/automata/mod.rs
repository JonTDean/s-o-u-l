//! Namespace for all automata‑related HUD panels.

pub mod show_active_automata;
pub mod spawn_panel;
pub mod selection_arrow;
pub mod plugin;               // keep plugin separate

pub use plugin::AutomataPanelPlugin;   // re‑export for convenience
