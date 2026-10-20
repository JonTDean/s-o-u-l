use serde::{Deserialize, Serialize};

/// Deterministic density fixtures; Ring is a ring pattern, not qualified Orbium.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeniaSeed {
    #[default]
    Blob,
    Ring,
    /// Published polynomial XY Orbium.
    Orbium,
}

/// Seed a bounded volume; small grids remain safe and outside cells stay zero.
pub fn seed(size: [u32; 3], pattern: LeniaSeed) -> Vec<u8> {
    if pattern == LeniaSeed::Orbium {
        return super::continuous::orbium::seed(size)
            .iter()
            .map(|v| (v * 255.0).round() as u8)
            .collect();
    }
    let [width, height, depth] = size;
    let mut cells = vec![0; (width * height * depth) as usize];
    let radius = (width.min(height) as f32 / 6.0).max(1.0);
    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - (width / 2) as f32;
                let dy = y as f32 - (height / 2) as f32;
                let dz = if depth == 1 {
                    0.0
                } else {
                    (z as f32 - (depth / 2) as f32) * 1.5
                };
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                let inside =
                    distance <= radius && (pattern == LeniaSeed::Blob || distance >= radius * 0.65);
                if inside {
                    cells[((z * height + y) * width + x) as usize] = 180;
                }
            }
        }
    }
    cells
}
