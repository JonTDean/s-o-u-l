//! Snapshot diffusion, replenishment and two-pool occupancy gating.
use super::GridSession;
use crate::workbench::experiment::config::energy::Energy;
use glam::IVec3;
use simulation_kernel::core::cell::CellState;
impl GridSession {
    pub fn reset_energy(&mut self, config: Option<&Energy>) {
        self.energy = config.map(|e| vec![e.initial; self.grid.voxels.len()]);
    }
    pub fn valid_energy(&self, config: Option<&Energy>) -> bool {
        match (&self.energy, config) {
            (None, None) => true,
            (Some(field), Some(e)) => {
                field.len() == self.grid.voxels.len()
                    && field
                        .iter()
                        .all(|p| p.iter().zip(e.capacity).all(|(v, cap)| *v <= cap))
            }
            _ => false,
        }
    }
    pub(super) fn gate_energy(&mut self, e: &Energy, prior: &[u8]) {
        let previous = self.energy.as_ref().expect("configured energy field");
        let size = self.grid.size;
        let mut next = previous.clone();
        for (index, pools) in next.iter_mut().enumerate() {
            let pos = IVec3::new(
                (index as u32 % size.x) as i32,
                (index as u32 / size.x) as i32,
                0,
            );
            for (pool, value) in pools.iter_mut().enumerate() {
                let sum: u32 = [IVec3::X, IVec3::NEG_X, IVec3::Y, IVec3::NEG_Y]
                    .iter()
                    .filter_map(|offset| self.boundary.coordinate(pos + offset, size))
                    .map(|p| u32::from(previous[self.grid.idx(p)][pool]))
                    .sum();
                let d = u32::from(e.diffusion[pool]);
                let mixed = (u32::from(previous[index][pool]) * (4000 - 4 * d) + sum * d) / 4000;
                *value = ((mixed as u16).saturating_sub(e.decay[pool]) + e.recharge[pool])
                    .min(e.capacity[pool]);
            }
            let costs = if prior[index] == 0 {
                e.birth_cost
            } else {
                e.survival_cost
            };
            let cell = &mut self.grid.voxels[index];
            if matches!(cell.state, CellState::Alive(1..)) && (0..2).all(|p| pools[p] >= costs[p]) {
                for p in 0..2 {
                    pools[p] -= costs[p];
                }
            } else {
                cell.state = CellState::Dead;
            }
        }
        self.energy = Some(next);
    }
}
