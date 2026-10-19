//! Bounded cellular sessions; elementary rules retain a scrolling time history.
use super::catalog::Kind;
use crate::automata::{
    classical::regular::wolfram_1d::rules::{rule30::Rule30, rule110::Rule110},
    dynamical::particle::{
        hpp::{E, HPPRule, W},
        life::{Life, seed_life},
    },
};
use glam::{IVec3, UVec3};
use simulation_kernel::{
    AutomatonRule,
    core::{cell::CellState, dim::Dim},
    grid::{DenseGrid, GridBackend},
    stepper::dense::step_dense,
};
use std::collections::VecDeque;

/// A finite grid with dead boundaries and snapshot update semantics.
pub struct GridSession {
    pub grid: DenseGrid,
    pub history: VecDeque<Vec<u8>>,
    kind: Kind,
}
impl GridSession {
    /// Initializes a single-cell elementary seed, Life glider, or colliding gas.
    pub fn new(kind: Kind) -> Self {
        let line = matches!(kind, Kind::Rule30 | Kind::Rule110);
        let mut grid = DenseGrid::blank(UVec3::new(64, if line { 1 } else { 64 }, 1));
        match kind {
            Kind::Rule30 | Kind::Rule110 => {
                grid.set_state(IVec3::new(32, 0, 0), CellState::Alive(255))
            }
            Kind::Life => {
                let mut backend = GridBackend::Dense(grid);
                seed_life(&mut backend);
                let GridBackend::Dense(seeded) = backend else {
                    unreachable!()
                };
                grid = seeded;
            }
            Kind::Hpp => {
                for y in 20..44 {
                    grid.set_state(IVec3::new(24, y, 0), CellState::Alive(E));
                    grid.set_state(IVec3::new(40, y, 0), CellState::Alive(W));
                }
            }
            _ => unreachable!("grid kind required"),
        }
        let mut session = Self {
            grid,
            history: VecDeque::new(),
            kind,
        };
        session.record();
        session
    }

    /// Advances from a complete prior grid, then retains at most 64 time rows.
    pub fn step(&mut self) {
        let rule: &dyn AutomatonRule<D = Dim> = match self.kind {
            Kind::Rule30 => &Rule30,
            Kind::Rule110 => &Rule110,
            Kind::Life => &Life,
            Kind::Hpp => &HPPRule,
            _ => unreachable!(),
        };
        step_dense(&mut self.grid, rule, &serde_json::Value::Null);
        self.record();
    }

    /// Raw state bytes; HPP direction bits remain intact for numerical checks.
    pub fn bytes(&self) -> Vec<u8> {
        self.grid
            .voxels
            .iter()
            .map(|cell| match cell.state {
                CellState::Dead => 0,
                CellState::Alive(value) => value,
            })
            .collect()
    }

    fn record(&mut self) {
        if self.grid.size.y == 1 {
            self.history.push_back(self.bytes());
            if self.history.len() > 64 {
                self.history.pop_front();
            }
        }
    }
}

#[cfg(test)]
mod tests;
