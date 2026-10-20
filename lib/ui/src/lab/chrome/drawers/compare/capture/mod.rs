//! Comparison snapshots are pinned to channel generations and completed ticks.
use engine_core::automata::channel::ChannelId;
use models::workbench::operations::channels::analysis::Field;
use simulation_kernel::core::lenia::{LeniaParams, continuous::Kernel};
mod field;
mod poll;
pub use poll::poll;
#[derive(Clone, Debug)]
pub struct Stamp {
    pub id: ChannelId,
    pub epoch: u64,
    pub tick: u64,
    pub params: Option<LeniaParams>,
    pub kernel: Option<Kernel>,
}
#[derive(Clone, Debug)]
pub struct Frame {
    pub stamp: Stamp,
    pub field: Field,
}
#[derive(Default)]
pub struct Capture {
    pub request: Option<bool>,
    pub pending: Option<Pending>,
    pub result: Option<[Frame; 2]>,
}
pub struct Pending {
    pub expected: [Stamp; 2],
    pub started: std::time::Instant,
}
impl Stamp {
    pub fn from_cpu(s: &models::workbench::session::Session) -> Self {
        Self {
            id: s.channel,
            epoch: s.epoch,
            tick: s.ticks,
            params: None,
            kernel: None,
        }
    }
    pub fn from_gpu(id: ChannelId, s: &engine_core::automata::lenia::LeniaSimulation) -> Self {
        Self {
            id,
            epoch: s.epoch,
            tick: s.target_tick,
            params: Some(s.params),
            kernel: s.kernel,
        }
    }
}
fn begin(
    state: &mut super::Comparison,
    step: bool,
    lookup: impl Fn(ChannelId) -> Result<Stamp, String>,
    commands: &mut bevy::prelude::EventWriter<engine_core::automata::channel::ChannelCommand>,
) -> Result<(), String> {
    use engine_core::automata::channel::{ChannelAction, ChannelCommand};
    let (left, right) = state.left.zip(state.right).ok_or("Choose two channels")?;
    if left == right {
        return Err("Choose two different channels".into());
    }
    let mut expected = [lookup(left)?, lookup(right)?];
    if step {
        for stamp in &mut expected {
            stamp.tick = stamp.tick.checked_add(1).ok_or("Tick limit reached")?;
        }
        for id in [left, right] {
            commands.write(ChannelCommand {
                id,
                action: ChannelAction::Step,
            });
        }
    }
    state.capture.pending = Some(Pending {
        expected,
        started: std::time::Instant::now(),
    });
    state.status = "Waiting for both completed states…".into();
    Ok(())
}
#[cfg(test)]
mod tests;
