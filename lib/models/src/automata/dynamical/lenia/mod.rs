//! Continuous Lenia rule (quantised 0–255 levels, 3×3 Gaussian kernel).

use glam::IVec3;
use simulation_kernel::{
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
    grid::GridBackend,
    AutomatonRule,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod plugin;

/* ───────────────── parameters ───────────────── */
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LeniaParams { pub mu: f32, pub sigma: f32, pub dt: f32 }
impl Default for LeniaParams { fn default() -> Self { Self { mu: 0.30, sigma: 0.06, dt: 0.05 } } }

/* ───────── 3×3 Gaussian kernel (XY plane) ───────── */
const KERNEL: [[f32; 3]; 3] = [
    [0.058_549_83, 0.096_532_35, 0.058_549_83],
    [0.096_532_35, 0.159_154_94, 0.096_532_35],
    [0.058_549_83, 0.096_532_35, 0.058_549_83],
];

/* ───────────────── rule type ───────────────── */
#[derive(Clone)]
pub struct LeniaRule;
impl LeniaRule {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim> + Send + Sync> {
        std::sync::Arc::new(Self)
    }
    #[inline] fn rho(cs: CellState) -> f32 { matches!(cs, CellState::Alive(level) if level > 0)
        .then(|| if let CellState::Alive(l) = cs { l } else { 0 } as f32 / 255.0).unwrap_or(0.0) }
    #[inline] fn to_state(rho: f32) -> CellState {
        if rho == 0.0 { CellState::Dead } else { CellState::Alive((rho * 255.0).round() as u8) }
    }
}

impl AutomatonRule for LeniaRule {
    type D = Dim;

    fn next_state(&self, ctx: CellCtx<Self::D>, params: &Value) -> CellOutcome {
        let p: LeniaParams = serde_json::from_value(params.clone()).unwrap_or_default();
        /* 1 — convolution on XY plane (neighbours with z=0) */
        let mut u = Self::rho(ctx.self_state) * KERNEL[1][1];
        for (i, off) in Dim::NEIGHBOUR_OFFSETS.iter().filter(|o| o.z == 0).enumerate() {
            let dx = (off.x + 1) as usize;
            let dy = (off.y + 1) as usize;
            u += Self::rho(ctx.neighbourhood[i]) * KERNEL[dy][dx];
        }
        /* 2 — bell-shaped growth */
        let g = 2.0 * (-(u - p.mu).powi(2) / (2.0 * p.sigma.powi(2))).exp() - 1.0;
        /* 3 — Euler step */
        let mut rho = Self::rho(ctx.self_state);
        rho = (rho + p.dt * g).clamp(0.0, 1.0);
        /* 4 — quantise back */
        let next_state = Self::to_state(rho);
        if next_state == ctx.self_state {
            CellOutcome::Unchanged
        } else {
            CellOutcome::Next { state: next_state, memory: ctx.memory.clone() }
        }
    }
}

/* ─────────────── seed helpers ─────────────── */
pub fn seed_lenia(grid: &mut GridBackend) {
    match grid {
        GridBackend::Dense(g) => {
            let centre = IVec3::new(g.size.x as i32 / 2, g.size.y as i32 / 2, 0);
            for y in -7..=7 {
                for x in -7..=7 {
                    if x * x + y * y <= 49 {
                        let idx = g.idx(centre + IVec3::new(x, y, 0));
                        g.voxels[idx].state = CellState::Alive(180);
                    }
                }
            }
        }
        GridBackend::Sparse(s) => { s.set_state(IVec3::ZERO, CellState::Alive(180)); }
    }
}

pub fn seed_orbium(grid: &mut GridBackend) {
    if let GridBackend::Dense(g) = grid {
        let centre = IVec3::new(g.size.x as i32 / 2, g.size.y as i32 / 2, 0);
        for y in -12..=12 {
            for x in -12..=12 {
                let r = ((x * x + y * y) as f32).sqrt();
                let idx = g.idx(centre + IVec3::new(x, y, 0));
                if (8.0..=10.0).contains(&r)       { g.voxels[idx].state = CellState::Alive(220); }
                else if (6.5..=7.5).contains(&r)  { g.voxels[idx].state = CellState::Alive(160); }
            }
        }
    }
}
