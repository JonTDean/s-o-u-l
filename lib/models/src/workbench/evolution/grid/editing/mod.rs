//! Paused cellular authoring with bounded stroke-level history.
use super::GridSession;
use crate::workbench::{
    catalog::Kind,
    session::{Model, Session},
};
use simulation_kernel::core::cell::CellState;
mod history;
pub use history::History;
impl Session {
    /// A stroke edits the current field into a new tick-zero reset seed.
    pub fn paint(
        &mut self,
        points: &[[i32; 2]],
        value: u8,
        start: bool,
    ) -> Result<(), &'static str> {
        if !self.paused {
            return Err("Pause before editing the world");
        }
        let Model::Grid(grid) = &mut self.model else {
            return Err("Painting requires a cellular grid");
        };
        if points.is_empty()
            || points.len() > 4096
            || points.iter().any(|p| {
                p[0] < 0
                    || p[1] < 0
                    || p[0] >= grid.grid.size.x as i32
                    || p[1] >= grid.grid.size.y as i32
            })
        {
            return Err("Paint stroke is outside the world");
        }
        if if self.kind == Kind::Hpp {
            value > 15
        } else {
            value != 0 && value != 255
        } {
            return Err("Invalid paint value");
        }
        if start {
            self.edits.remember(
                grid,
                self.ticks,
                self.elapsed,
                self.settings.world.seed.clone(),
            );
        }
        for [x, y] in points {
            grid.grid
                .set_state(glam::IVec3::new(*x, *y, 0), CellState::from(value));
        }
        grid.reset_energy(self.settings.energy.as_ref());
        self.settings.world.seed = Some(grid.bytes());
        grid.history.clear();
        grid.record();
        self.ticks = 0;
        self.elapsed = 0.0;
        self.epoch += 1;
        Ok(())
    }
    pub fn undo_paint(&mut self, redo: bool) -> Result<(), &'static str> {
        if !self.paused {
            return Err("Pause before undo or redo");
        }
        let Model::Grid(grid) = &mut self.model else {
            return Err("Undo requires a cellular grid");
        };
        self.edits.restore(
            redo,
            grid,
            &mut self.ticks,
            &mut self.elapsed,
            &mut self.settings.world.seed,
        )?;
        self.epoch += 1;
        Ok(())
    }
}
#[cfg(test)]
mod tests;
