//! HPP lattice-gas automaton (2D square grid, 4 velocity directions).
//!
//! Each live cell packs four velocity bits as **0b0000 N E S W** (bit = 1 indicates a particle moving in that direction).
//! The rule performs local *collision* and *streaming* in one step (no neighbor writes, making it trivially parallel).
//!
//! References: 
//! • Hardy–Pomeau–de Pazzis model overview【1】  
//! • Implementation patterns in lattice-gas codes【2】

use engine_core::core::{cell::CellState, dim::Dim2, AutomatonRule, CellCtx, CellOutcome};
use serde_json::Value;

/// Four orthogonal velocity directions encoded as bit masks (N, E, S, W).
impl HPPRule {
    pub const N: u8 = 0b1000;
    pub const E: u8 = 0b0100;
    pub const S: u8 = 0b0010;
    pub const W: u8 = 0b0001;
}

/// Zero-sized struct implementing the HPP rule.
#[derive(Clone)]
pub struct HPPRule;
impl HPPRule {
    /// Boxed trait object for registration.
    #[inline]
    pub fn boxed() -> std::sync::Arc<dyn AutomatonRule<D = Dim2> + Send + Sync> {
        std::sync::Arc::new(Self)
    }

    /// Extract velocity bits from a cell state (dead = 0).
    #[inline(always)]
    fn bits(cs: CellState) -> u8 {
        if let CellState::Alive(b) = cs { b } else { 0 }
    }

    /// Pack velocity bits into a CellState (0 = dead).
    #[inline(always)]
    fn to_state(bits: u8) -> CellState {
        if bits == 0 { CellState::Dead } else { CellState::Alive(bits) }
    }
}

impl AutomatonRule for HPPRule {
    type D = Dim2;

    fn next_state(&self, ctx: CellCtx<Self::D>, _params: &Value) -> CellOutcome {
        // 1. Determine incoming bits from neighbors
        let mut incoming = 0;
        if Self::bits(ctx.neighbourhood[1]) & HPPRule::S != 0 { incoming |= HPPRule::N; }
        if Self::bits(ctx.neighbourhood[7]) & HPPRule::N != 0 { incoming |= HPPRule::S; }
        if Self::bits(ctx.neighbourhood[3]) & HPPRule::E != 0 { incoming |= HPPRule::W; }
        if Self::bits(ctx.neighbourhood[5]) & HPPRule::W != 0 { incoming |= HPPRule::E; }

        // 2. Collision: if two particles head-on, they collide and swap directions
        let mut post = incoming;
        // Vertical head-on (N+S -> become E+W)
        if incoming & (HPPRule::N | HPPRule::S) == (HPPRule::N | HPPRule::S)
            && incoming & (HPPRule::E | HPPRule::W) == 0
        {
            post &= !(HPPRule::N | HPPRule::S);
            post |=  HPPRule::E | HPPRule::W;
        }
        // Horizontal head-on (E+W -> become N+S)
        if incoming & (HPPRule::E | HPPRule::W) == (HPPRule::E | HPPRule::W)
            && incoming & (HPPRule::N | HPPRule::S) == 0
        {
            post &= !(HPPRule::E | HPPRule::W);
            post |=  HPPRule::N | HPPRule::S;
        }

        // 3. The result `post` bits represent the new state at this cell after collision and streaming.
        let new_bits = post;
        if new_bits == Self::bits(ctx.self_state) {
            CellOutcome::Unchanged
        } else {
            CellOutcome::Next { state: Self::to_state(new_bits), memory: ctx.memory.clone() }
        }
    }
}

/* ───────────────────────────── seeding function ───────────────────────────── */

use bevy::math::IVec2;
use engine_core::engine::grid::GridBackend::{Dense, Sparse};

/// Seed pattern for HPP: a cross of streams at the center to demonstrate collisions.
///
/// This seeds one cell at the center with particles moving in all four directions (N, E, S, W).
pub fn seed_hpp(grid: &mut engine_core::engine::grid::GridBackend) {
    match grid {
        Dense(g) => {
            let centre = IVec2::new(g.size.x as i32 / 2, g.size.y as i32 / 2);
            let idx = g.idx(centre);
            g.cells[idx].state = CellState::Alive(HPPRule::N | HPPRule::E | HPPRule::S | HPPRule::W);
        }
        Sparse(s) => {
            s.set_state(IVec2::ZERO, CellState::Alive(HPPRule::N | HPPRule::E | HPPRule::S | HPPRule::W));
        }
    }
}
