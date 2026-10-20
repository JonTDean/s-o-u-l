use super::GridSession;
#[derive(Default)]
pub struct History {
    undo: Vec<Revision>,
    redo: Vec<Revision>,
}
struct Revision {
    grid: GridSession,
    tick: u64,
    elapsed: f32,
    seed: Option<Vec<u8>>,
}
impl History {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub(super) fn remember(
        &mut self,
        grid: &GridSession,
        tick: u64,
        elapsed: f32,
        seed: Option<Vec<u8>>,
    ) {
        if self.undo.len() == 16 {
            self.undo.remove(0);
        }
        self.undo.push(Revision {
            grid: grid.clone(),
            tick,
            elapsed,
            seed,
        });
        self.redo.clear();
    }
    pub(super) fn restore(
        &mut self,
        redo: bool,
        grid: &mut GridSession,
        tick: &mut u64,
        elapsed: &mut f32,
        seed: &mut Option<Vec<u8>>,
    ) -> Result<(), &'static str> {
        let (source, target) = if redo {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        let Some(mut previous) = source.pop() else {
            return Err("No edit to undo or redo");
        };
        std::mem::swap(grid, &mut previous.grid);
        std::mem::swap(tick, &mut previous.tick);
        std::mem::swap(elapsed, &mut previous.elapsed);
        std::mem::swap(seed, &mut previous.seed);
        target.push(previous);
        Ok(())
    }
}
