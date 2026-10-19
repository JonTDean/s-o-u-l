use crate::workbench::{catalog::Kind, experiment::config::Settings};
use simulation_kernel::{
    AutomatonRule,
    core::{
        cell::{CellCtx, CellOutcome, CellState},
        dim::{Dim, Dimensionality},
    },
};

// The kernel trait is 'static; copy the small numerical contract into each rule.
pub(crate) struct CellularRule {
    pub elementary: Option<u8>,
    pub birth: u16,
    pub survival: u16,
}
impl CellularRule {
    pub fn new(kind: Kind, settings: &Settings) -> Self {
        Self {
            elementary: matches!(kind, Kind::Rule30 | Kind::Rule110).then_some(settings.elementary),
            birth: settings.birth,
            survival: settings.survival,
        }
    }
}
impl AutomatonRule for CellularRule {
    type D = Dim;
    fn next_state(&self, ctx: CellCtx<Dim>, _: &serde_json::Value) -> CellOutcome {
        let alive = |state| matches!(state, CellState::Alive(1..));
        let next = if let Some(number) = self.elementary {
            let pattern = (u8::from(alive(ctx.neighbourhood[12])) << 2)
                | (u8::from(alive(ctx.self_state)) << 1)
                | u8::from(alive(ctx.neighbourhood[13]));
            number & (1 << pattern) != 0
        } else {
            let count = Dim::NEIGHBOUR_OFFSETS
                .iter()
                .zip(ctx.neighbourhood)
                .filter(|(p, s)| p.z == 0 && alive(**s))
                .count();
            let mask = if alive(ctx.self_state) {
                self.survival
            } else {
                self.birth
            };
            mask & (1 << count) != 0
        };
        CellOutcome::Next {
            state: CellState::from(if next { 255 } else { 0 }),
            memory: ctx.memory.clone(),
        }
    }
}
