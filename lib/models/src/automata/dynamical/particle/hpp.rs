//! HPP lattice-gas automaton on a 3-D grid (motion constrained to z = 0).

use glam::IVec3;
use simulation_kernel::{
    core::{cell::{CellCtx, CellOutcome, CellState}, dim::{Dim, Dimensionality}},
    grid::GridBackend,
    AutomatonRule,
};
use serde_json::Value;

/* ───────── velocity bit-masks ───────── */
pub const N: u8 = 0b1000;
pub const E: u8 = 0b0100;
pub const S: u8 = 0b0010;
pub const W: u8 = 0b0001;

/* ───────── rule type ───────── */
#[derive(Clone)]
pub struct HPPRule;
impl HPPRule {
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim> + Send + Sync> {
        std::sync::Arc::new(Self)
    }
    #[inline] fn bits(cs: CellState) -> u8 { if let CellState::Alive(b) = cs { b } else { 0 } }
    #[inline] fn to_state(bits: u8) -> CellState { if bits == 0 { CellState::Dead } else { CellState::Alive(bits) } }
}

impl AutomatonRule for HPPRule {
    type D = Dim;

    fn next_state(&self, ctx: CellCtx<Self::D>, _params: &Value) -> CellOutcome {
        /* 1 — gather incoming (XY plane only) */
        let mut incoming = 0;
        if Self::bits(ctx.neighbourhood.iter().find(|&&o| o == CellState::Dead).copied().unwrap_or(CellState::Dead)) & S != 0 { incoming |= N; }
        // Use explicit offsets for clarity
        let nb = ctx.neighbourhood;
        if Self::bits(nb[ Dim::NEIGHBOUR_OFFSETS.iter().position(|o| *o == IVec3::new( 0,  1, 0)).unwrap() ]) & S != 0 { incoming |= N; }
        if Self::bits(nb[ Dim::NEIGHBOUR_OFFSETS.iter().position(|o| *o == IVec3::new( 0, -1, 0)).unwrap() ]) & N != 0 { incoming |= S; }
        if Self::bits(nb[ Dim::NEIGHBOUR_OFFSETS.iter().position(|o| *o == IVec3::new( 1,  0, 0)).unwrap() ]) & W != 0 { incoming |= E; }
        if Self::bits(nb[ Dim::NEIGHBOUR_OFFSETS.iter().position(|o| *o == IVec3::new(-1,  0, 0)).unwrap() ]) & E != 0 { incoming |= W; }

        /* 2 — collisions */
        let mut post = incoming;
        if incoming & (N | S) == (N | S) && incoming & (E | W) == 0 {
            post &= !(N | S); post |= E | W;
        }
        if incoming & (E | W) == (E | W) && incoming & (N | S) == 0 {
            post &= !(E | W); post |= N | S;
        }

        let new_state = Self::to_state(post);
        if new_state == ctx.self_state {
            CellOutcome::Unchanged
        } else {
            CellOutcome::Next { state: new_state, memory: ctx.memory.clone() }
        }
    }
}

/* ───────────── seeding ───────────── */
pub fn seed_hpp(grid: &mut GridBackend) {
    match grid {
        GridBackend::Dense(g) => {
            let centre = IVec3::new(g.size.x as i32 / 2, g.size.y as i32 / 2, 0);
            let idx = g.idx(centre);
            g.voxels[idx].state = CellState::Alive(N | E | S | W);
        }
        GridBackend::Sparse(s) => {
            s.set_state(IVec3::ZERO, CellState::Alive(N | E | S | W));
        }
    }
}
