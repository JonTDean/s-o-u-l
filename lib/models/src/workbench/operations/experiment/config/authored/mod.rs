//! Portable binary Moore-neighborhood programs with explicit ordered transitions.
pub mod composition;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub mask: u16,
    pub pattern: u16,
    pub alive: bool,
}
/// Bit 0 is (-1,-1), bit 4 the centre, bit 8 (+1,+1); first match wins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub version: u32,
    pub name: String,
    pub default_alive: bool,
    pub transitions: Vec<Transition>,
}
impl Rule {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != 1 {
            return Err("Unsupported rule version");
        }
        if self.name.trim().is_empty()
            || self.name.len() > 64
            || self.name.chars().any(char::is_control)
        {
            return Err("Rule names require 1..64 bytes without control characters");
        }
        if self.transitions.len() > 512
            || self
                .transitions
                .iter()
                .any(|t| t.mask > 511 || t.pattern & !t.mask != 0)
        {
            return Err(
                "Use at most 512 transitions with nine-bit masks and patterns contained in masks",
            );
        }
        Ok(())
    }
    pub fn evaluate(&self, neighborhood: u16) -> bool {
        self.transitions
            .iter()
            .find(|t| neighborhood & t.mask == t.pattern)
            .map_or(self.default_alive, |t| t.alive)
    }
    pub fn life(birth: u16, survival: u16) -> Self {
        let transitions = (0u16..512)
            .filter_map(|pattern| {
                let count = (pattern & !(1 << 4)).count_ones();
                let mask = if pattern & (1 << 4) == 0 {
                    birth
                } else {
                    survival
                };
                (mask & (1 << count) != 0).then_some(Transition {
                    mask: 511,
                    pattern,
                    alive: true,
                })
            })
            .collect();
        Self {
            version: 1,
            name: "Authored Life".into(),
            default_alive: false,
            transitions,
        }
    }
    pub fn copy_west() -> Self {
        Self {
            version: 1,
            name: "Copy west".into(),
            default_alive: false,
            transitions: vec![Transition {
                mask: 8,
                pattern: 8,
                alive: true,
            }],
        }
    }
}
