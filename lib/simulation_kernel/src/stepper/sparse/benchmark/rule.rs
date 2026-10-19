use super::*;
pub(super) struct Life;
impl AutomatonRule for Life {
    type D = Dim;
    fn next_state(&self, c: CellCtx<Dim>, _: &Value) -> CellOutcome {
        let n = Dim::NEIGHBOUR_OFFSETS
            .iter()
            .zip(c.neighbourhood)
            .filter(|(p, s)| p.z == 0 && **s != CellState::Dead)
            .count();
        CellOutcome::Next {
            state: if n == 3 || (n == 2 && c.self_state != CellState::Dead) {
                CellState::Alive(255)
            } else {
                CellState::Dead
            },
            memory: c.memory.clone(),
        }
    }
}
