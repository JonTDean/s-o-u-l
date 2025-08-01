//! Data‑only “model” layer for the main‑menu screens.

use serde::{Deserialize, Serialize};

/// Dense versus sparse grid back‑end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridType {
    Dense,
    Sparse,
}

/// RGBA color (0–255 per channel) stored in the model without egui deps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Rgba {
    fn default() -> Self {
        Rgba {
            r: 40,
            g: 40,
            b: 60,
            a: 255,
        }
    }
}

/// Draft parameters the user edits when creating a new scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDraft {
    pub name:               String,
    pub width:              u32,
    pub height:             u32,
    pub depth:              u32,
    pub voxel_size:          f32,
    pub grid_type:          GridType,
    pub bg_color:           Rgba,
    pub selected_classical: Vec<String>,
    /// *Exactly one* dynamical automaton (or `None`)
    pub selected_dynamical: Option<String>,
}

impl Default for ScenarioDraft {
    fn default() -> Self {
        Self {
            name: "MyScenario".into(),
            width:  64,
            height: 64,
            depth:  64,
            voxel_size: 16.0,
            grid_type: GridType::Dense,
            bg_color: Rgba::default(),
            selected_classical: Vec::new(),
            selected_dynamical: None,
        }
    }
}
