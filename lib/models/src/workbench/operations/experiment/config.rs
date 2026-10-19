use crate::workbench::catalog::Kind;
use serde::{Deserialize, Serialize};

/// Initial condition; changing it requires an explicit reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern {
    Default,
    Single,
    Glider,
    Blinker,
    Noise,
}

/// Serializable controls, validated before entering a live session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub rate_hz: f32,
    pub pattern: Pattern,
    pub random_seed: u64,
    pub elementary: u8,
    pub birth: u16,
    pub survival: u16,
    pub cohesion: f32,
    pub alignment: f32,
    pub separation: f32,
    pub separation_distance: f32,
    pub agent_count: usize,
    pub evaporation: f32,
    pub deposit: f32,
    pub exploration: f32,
}
impl Settings {
    /// Preserve the qualified fixture's defaults for each family.
    pub fn for_kind(kind: Kind) -> Self {
        Self {
            rate_hz: 10.0,
            pattern: Pattern::Default,
            random_seed: 0x534f554c,
            elementary: if kind == Kind::Rule110 { 110 } else { 30 },
            birth: 1 << 3,
            survival: (1 << 2) | (1 << 3),
            cohesion: 0.002,
            alignment: 0.08,
            separation: 0.1,
            separation_distance: 4.0,
            agent_count: if kind == Kind::Ants { 32 } else { 24 },
            evaporation: 0.01,
            deposit: 1.0,
            exploration: 0.0,
        }
    }
    /// Initial conditions are stable until reset, unlike live rule parameters.
    pub fn same_initial_conditions(&self, other: &Self) -> bool {
        self.pattern == other.pattern
            && self.random_seed == other.random_seed
            && self.agent_count == other.agent_count
    }
}
