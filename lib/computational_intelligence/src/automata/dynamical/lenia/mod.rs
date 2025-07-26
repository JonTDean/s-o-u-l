//! Continuous Lenia rule (quantized 0–255 levels, 3×3 Gaussian kernel).

use bevy::prelude::IVec2;
use engine_core::core::{cell::CellState, dim::Dim2, AutomatonRule, CellCtx, CellOutcome, Dim};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod plugin;

/* ───────────────────── parameters ───────────────────── */
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LeniaParams {
    pub mu:    f32,
    pub sigma: f32,
    pub dt:    f32,
}
impl Default for LeniaParams {
    fn default() -> Self {
        // parameters tuned to produce the Orbium lifeform (peak at ρ ≃ 0.30)
        Self { mu: 0.30, sigma: 0.06, dt: 0.05 }
    }
}

/* ─────────────── 3×3 Gaussian kernel (Lenia's convolution mask) ─────────────── */
const KERNEL: [[f32; 3]; 3] = [
    [0.058_549_83, 0.096_532_35, 0.058_549_83],
    [0.096_532_35, 0.159_154_94, 0.096_532_35],
    [0.058_549_83, 0.096_532_35, 0.058_549_83],
];

/* ───────────────────── rule type ───────────────────── */
#[derive(Clone)]
pub struct LeniaRule;
impl LeniaRule {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim2> + Send + Sync> {
        std::sync::Arc::new(Self)
    }

    #[inline(always)]
    fn rho(cs: CellState) -> f32 {
        if let CellState::Alive(level) = cs {
            level as f32 / 255.0  // normalize 0-255 level to [0.0,1.0]
        } else {
            0.0
        }
    }
    #[inline(always)]
    fn to_state(rho: f32) -> CellState {
        if rho == 0.0 {
            CellState::Dead
        } else {
            CellState::Alive((rho * 255.0).round() as u8)
        }
    }
}

impl AutomatonRule for LeniaRule {
    type D = Dim2;

    fn next_state(&self, ctx: CellCtx<Self::D>, params: &Value) -> CellOutcome {
        // Deserialize parameters or use default
        let p: LeniaParams = serde_json::from_value(params.clone()).unwrap_or_default();
        /* 1 ─ compute convolution u (weighted sum of center + neighbours) */
        let mut u = Self::rho(ctx.self_state) * KERNEL[1][1];
        for (i, off) in Dim2::NEIGHBOUR_OFFSETS.iter().enumerate() {
            let dx = (off.x + 1) as usize;
            let dy = (off.y + 1) as usize;
            u += Self::rho(ctx.neighbourhood[i]) * KERNEL[dy][dx];
        }
        /* 2 ─ bell‑shaped growth function */
        let g = 2.0 * (-(u - p.mu).powi(2) / (2.0 * p.sigma.powi(2))).exp() - 1.0;
        /* 3 ─ Euler integration (update density) */
        let mut rho = Self::rho(ctx.self_state);
        rho = (rho + p.dt * g).clamp(0.0, 1.0);
        /* 4 ─ convert back to discrete state */
        let next_state = Self::to_state(rho);
        if next_state == ctx.self_state {
            CellOutcome::Unchanged
        } else {
            CellOutcome::Next { state: next_state, memory: ctx.memory.clone() }
        }
    }
}

/* ─────────────────── seed functions for Lenia ─────────────────── */

use engine_core::engine::grid::GridBackend::{Dense, Sparse};

/// Default seed pattern for Lenia: a solid circular blob of moderate density at the center.
pub fn seed_lenia(grid: &mut engine_core::engine::grid::GridBackend) {
    match grid {
        Dense(g) => {
            let cx = g.size.x as i32 / 2;
            let cy = g.size.y as i32 / 2;
            // populate a 7-cell radius filled disk
            for y in -7..=7 {
                for x in -7..=7 {
                    if (x * x + y * y) <= 49 {
                        let idx = g.idx(IVec2::new(cx + x, cy + y));
                        g.cells[idx].state = CellState::Alive(180);
                    }
                }
            }
        }
        Sparse(s) => {
            // in sparse case, just set a single cell at origin with a medium-high level
            s.set_state(IVec2::ZERO, CellState::Alive(180));
        }
    }
}

/// Alternate seed: the classic “Orbium” life-form from Lenia literature.
///
/// Orbium is a hollow ring with a slightly thicker rim. It spawns a self-sustaining rotating blob.
pub fn seed_orbium(grid: &mut engine_core::engine::grid::GridBackend) {
    if let Dense(g) = grid {
        let cx = g.size.x as i32 / 2;
        let cy = g.size.y as i32 / 2;
        for y in -12..=12 {
            for x in -12..=12 {
                let r = (x * x + y * y) as f32;
                // ring radius ~9 with thickness
                if (8.0..=10.0).contains(&r.sqrt()) {
                    let idx = g.idx(IVec2::new(cx + x, cy + y));
                    g.cells[idx].state = CellState::Alive(220);   // bright rim
                } else if (6.5..=7.5).contains(&r.sqrt()) {
                    let idx = g.idx(IVec2::new(cx + x, cy + y));
                    g.cells[idx].state = CellState::Alive(160);   // inner halo
                }
            }
        }
    }
}
