//! Comparison snapshots are pinned to channel generations and completed ticks.
use engine_core::automata::channel::ChannelId;
use simulation_kernel::core::lenia::{LeniaParams,continuous::Kernel};
use models::workbench::operations::channels::analysis::Field;
mod poll;
mod field;
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
pub struct Frame { pub stamp: Stamp, pub field: Field }
#[derive(Default)]
pub struct Capture {
    pub request: Option<bool>,
    pub pending: Option<Pending>,
    pub result: Option<[Frame;2]>,
}
pub struct Pending {
    pub expected: [Stamp;2],
    pub started: std::time::Instant,
}
impl Stamp {
    pub fn from_cpu(s: &models::workbench::session::Session) -> Self {
        Self { id:s.channel, epoch:s.epoch, tick:s.ticks, params:None,kernel:None }
    }
    pub fn from_gpu(id:ChannelId,s:&engine_core::automata::lenia::LeniaSimulation) -> Self {
        Self { id, epoch:s.epoch,tick:s.target_tick,params:Some(s.params),kernel:s.kernel }
    }
}
