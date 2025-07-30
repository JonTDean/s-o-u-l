use std::sync::Arc;
use glam::Vec2;
use bevy::color::Color;
use serde_json::Value;
use simulation_kernel::{core::dim::Dim2, grid::GridBackend, AutomatonRule};

use crate::events::AutomatonId;


/* ──────────────────────────────────────────────────────────────────── */
/* Automata registry                                                   */
/* ──────────────────────────────────────────────────────────────────── */
pub struct AutomatonInfo {
    pub id: AutomatonId,
    pub name: String,
    pub rule: Arc<dyn AutomatonRule<D = Dim2> + Send + Sync>,
    pub params: Value,
    pub seed_fn: Option<fn(&mut GridBackend)>,
    pub grid: GridBackend,
    pub dimension: u8,
    pub cell_size: f32,
    pub background_color: Color,
    pub palette: Option<Vec<Color>>,
    pub world_offset: Vec2,
}

/// Manual `Debug` impl (the rule & grid are not `Debug` themselves).
impl std::fmt::Debug for AutomatonInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutomatonInfo")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("dimension", &self.dimension)
            .field("cell_size", &self.cell_size)
            .finish()
    }
}
