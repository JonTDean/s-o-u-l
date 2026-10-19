//! Metric contract shared by CPU sources and the GPU adapter (completed in Task 2).
use super::ChannelId;
use bevy::prelude::Resource;

/// Metric state keyed by channel.
#[derive(Resource, Debug, Default)]
pub struct Metrics(pub std::collections::HashMap<ChannelId, ()>);

impl Metrics {
    /// Keep only channels for which `open` holds.
    pub fn retain(&mut self, open: impl Fn(ChannelId) -> bool) {
        self.0.retain(|id, _| open(*id));
    }
}
