//! Continuous Lenia rule (quantised 0–255 levels, 3×3 Gaussian kernel).

use glam::IVec3;
use serde_json::Value;
use simulation_kernel::{
    AutomatonRule,
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::GridBackend,
};

pub mod plugin;
#[cfg(test)]
mod tests;

pub use simulation_kernel::core::lenia::LeniaParams;
use simulation_kernel::core::lenia::{KERNEL, next_density};

/* ───────────────── rule type ───────────────── */
#[derive(Clone)]
pub struct LeniaRule;
impl LeniaRule {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim> + Send + Sync> {
        std::sync::Arc::new(Self)
    }
    #[inline]
    fn rho(cs: CellState) -> f32 {
        matches!(cs, CellState::Alive(level) if level > 0)
            .then(|| if let CellState::Alive(l) = cs { l } else { 0 } as f32 / 255.0)
            .unwrap_or(0.0)
    }
}

impl AutomatonRule for LeniaRule {
    type D = Dim;

    fn next_state(&self, ctx: CellCtx<Self::D>, params: &Value) -> CellOutcome {
        let p: LeniaParams = serde_json::from_value(params.clone()).unwrap_or_default();
        let Ok(p) = p.validate() else {
            return CellOutcome::Unchanged;
        };
        /* 1 — convolution on XY plane (neighbours with z=0) */
        let mut u = Self::rho(ctx.self_state) * KERNEL[1][1];
        for (i, off) in Dim::NEIGHBOUR_OFFSETS
            .iter()
            .enumerate()
            .filter(|(_, o)| o.z == 0)
        {
            let dx = (off.x + 1) as usize;
            let dy = (off.y + 1) as usize;
            u += Self::rho(ctx.neighbourhood[i]) * KERNEL[dy][dx];
        }
        let density = match ctx.self_state {
            CellState::Dead => 0,
            CellState::Alive(v) => v,
        };
        let next_state = CellState::from(next_density(density, u, p));
        if next_state == ctx.self_state {
            CellOutcome::Unchanged
        } else {
            CellOutcome::Next {
                state: next_state,
                memory: ctx.memory.clone(),
            }
        }
    }
}

/* ─────────────── seed helpers ─────────────── */
pub fn seed_lenia(grid: &mut GridBackend) {
    seed_pattern(grid, simulation_kernel::core::lenia::LeniaSeed::Blob);
}

pub fn seed_orbium(grid: &mut GridBackend) {
    seed_pattern(grid, simulation_kernel::core::lenia::LeniaSeed::Ring);
}

fn seed_pattern(grid: &mut GridBackend, pattern: simulation_kernel::core::lenia::LeniaSeed) {
    match grid {
        GridBackend::Dense(g) => {
            let cells = simulation_kernel::core::lenia::seed(g.size.to_array(), pattern);
            for (cell, density) in g.voxels.iter_mut().zip(cells) {
                cell.state = CellState::from(density);
            }
        }
        GridBackend::Sparse(s) => s.set_state(IVec3::ZERO, CellState::Alive(180)),
    }
}
