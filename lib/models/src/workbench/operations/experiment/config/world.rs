//! Cellular dimensions, boundary topology and an optional authored reset seed.
use crate::workbench::catalog::Kind;
use glam::UVec3;
use simulation_kernel::core::boundary::Boundary;
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub width: u32,
    pub height: u32,
    pub boundary: Boundary,
    pub seed: Option<Vec<u8>>,
}
impl Default for World {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            boundary: Boundary::Dead,
            seed: None,
        }
    }
}
impl World {
    pub fn size(&self, kind: Kind) -> UVec3 {
        UVec3::new(
            self.width,
            if matches!(kind, Kind::Rule30 | Kind::Rule110) {
                1
            } else {
                self.height
            },
            1,
        )
    }
    pub fn validate(&self, kind: Kind) -> Result<(), &'static str> {
        if !(8..=256).contains(&self.width) || !(8..=256).contains(&self.height) {
            return Err("World dimensions must be 8..256 cells");
        }
        if let Some(seed) = &self.seed {
            let size = self.size(kind);
            if seed.len() != (size.x * size.y) as usize
                || seed.iter().any(|v| {
                    if kind == Kind::Hpp {
                        *v > 15
                    } else {
                        *v != 0 && *v != 255
                    }
                })
            {
                return Err("Authored seed must match the world and cell alphabet");
            }
        }
        Ok(())
    }
}
