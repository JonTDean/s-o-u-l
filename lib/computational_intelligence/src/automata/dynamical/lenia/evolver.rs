//! Evolver logic for Lenia – updates the world grid according to Lenia rules.

use engine_core::core::world::World2D;
use engine_core::core::cell::{CellState, CellMemory};
use super::kernel::Kernel;
use super::species::Orbium;

/// Performs one Lenia update step on the given world using the provided species parameters.
pub fn evolve_lenia(world: &mut World2D, species: &Orbium) {
    let Kernel { offsets, weights } = &species.kernel;
    let radius = species.radius;
    let width;
    let height;
    // Determine world dimensions
    if let engine_core::engine::grid::GridBackend::Dense(grid) = &mut world.backend {
        width = grid.size.x as i32;
        height = grid.size.y as i32;
        // Prepare a copy of current continuous state values
        let mut current_values = vec![0.0; (width * height) as usize];
        for (idx, cell) in grid.cells.iter().enumerate() {
            let val = if let CellState::Alive(lv) = cell.state {
                // If using discrete Alive level (0-255) as proxy for continuous
                lv as f32 / 255.0
            } else {
                0.0
            };
            // If there's a stored precise value in memory, use it instead
            let val = if let CellMemory::Number(n) = &cell.memory {
                n.as_f64().unwrap_or(val as f64) as f32
            } else {
                val
            };
            current_values[idx] = val;
        }
        // Buffer for next values
        let mut next_values = vec![0.0; current_values.len()];
        // Convolution and growth
        for (idx, cell) in grid.cells.iter_mut().enumerate() {
            let x = (idx as u32 % grid.size.x) as i32;
            let y = (idx as u32 / grid.size.x) as i32;
            // Sum weighted neighborhood within radius
            let mut accumulated = 0.0;
            for (off, w) in offsets.iter().zip(weights.iter()) {
                let nx = x + off.x;
                let ny = y + off.y;
                if nx < 0 || nx >= width || ny < 0 || ny >= height {
                    continue;
                }
                let n_index = (ny as u32 * grid.size.x + nx as u32) as usize;
                accumulated += current_values[n_index] * *w;
            }
            // Compute growth (using Orbium's growth parameters mu and sigma)
            let growth = species.growth_curve(accumulated);
            let current_val = current_values[idx];
            let new_val = (current_val + species.time_step * growth)
                .max(0.0).min(1.0);
            next_values[idx] = new_val;
        }
        // Apply the new continuous values to the world
        for (idx, cell) in grid.cells.iter_mut().enumerate() {
            let new_val = next_values[idx];
            // Update memory with precise value
            cell.memory = serde_json::Value::Number(serde_json::Number::from_f64(new_val as f64).unwrap());
            // Update discrete state for visualization (threshold or quantize)
            if new_val > 0.1 {
                // Quantize to 1..255
                let level = (new_val * 255.0).round().max(1.0) as u8;
                cell.state = CellState::Alive(level);
            } else {
                cell.state = CellState::Dead;
            }
        }
    } else {
        // Lenia is more naturally run on a dense grid, sparse grid handling is not implemented
        return;
    }
}
