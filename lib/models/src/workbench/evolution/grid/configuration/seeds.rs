use super::super::GridSession;
use crate::workbench::{
    catalog::Kind,
    experiment::config::{Pattern, Settings},
};
use glam::IVec3;
use rand::{Rng, SeedableRng, rngs::StdRng};
use simulation_kernel::core::cell::CellState;

impl GridSession {
    /// Build a validated initial condition without preserving old history.
    pub fn configured(kind: Kind, settings: &Settings) -> Self {
        let mut session = Self::with_size(kind, settings.world.size(kind));
        session.boundary = settings.world.boundary;
        if let Some(seed) = &settings.world.seed {
            for (cell, value) in session.grid.voxels.iter_mut().zip(seed) {
                cell.state = CellState::from(*value);
            }
            session.history.clear();
            session.record();
            return session;
        }
        if matches!(settings.pattern, Pattern::Default | Pattern::Glider) {
            return session;
        }
        for cell in &mut session.grid.voxels {
            cell.state = CellState::Dead;
        }
        let centre = (session.grid.size / 2).as_ivec3();
        match settings.pattern {
            Pattern::Single => session.grid.set_state(centre, CellState::Alive(255)),
            Pattern::Blinker => {
                for x in -1..=1 {
                    session
                        .grid
                        .set_state(centre + IVec3::new(x, 0, 0), CellState::Alive(255));
                }
            }
            Pattern::Noise => {
                let mut rng = StdRng::seed_from_u64(settings.random_seed);
                for cell in &mut session.grid.voxels {
                    let value = if rng.random_bool(0.2) {
                        if kind == Kind::Hpp {
                            rng.random_range(1..16)
                        } else {
                            255
                        }
                    } else {
                        0
                    };
                    cell.state = CellState::from(value);
                }
            }
            _ => {}
        }
        session.history.clear();
        session.record();
        session
    }
}
