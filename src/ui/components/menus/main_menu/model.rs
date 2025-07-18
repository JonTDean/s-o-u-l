//! Data-only “model” layer for the main-menu screens.

use serde::{Deserialize, Serialize};

/// Dense versus sparse grid back-end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridType {
    Dense,
    Sparse,
}

/// RGBA color (0–255 per channel) stored in the model without egui dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Rgba {
    fn default() -> Self {
        // Dark blue background to match the existing style
        Rgba { r: 40, g: 40, b: 60, a: 255 }
    }
}

/// Draft parameters the user edits when creating a new scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDraft {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Size of each cell in pixels (controls the simulation’s visual resolution).
    pub cell_size: f32,
    /// Dense (contiguous) or sparse (hash-map) grid representation.
    pub grid_type: GridType,
    /// Background color for the grid.
    pub bg_color: Rgba,
}

impl Default for ScenarioDraft {
    fn default() -> Self {
        Self {
            name:       "MyScenario".into(),
            width:      64,
            height:     64,
            cell_size:  16.0,
            grid_type:  GridType::Dense,
            bg_color:   Rgba::default(),
        }
    }
}

/// Placeholder data structure for the load-scenario workflow (file path, preview metadata, etc.).
#[derive(Debug, Default)] pub struct LoadScenarioData;

/// Persistent user settings from the **Options** screen.
#[derive(Debug, Default)] pub struct UiSettings { pub font_size: f32 }