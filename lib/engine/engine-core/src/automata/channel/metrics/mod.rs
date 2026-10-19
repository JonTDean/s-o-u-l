//! Metric contract shared by CPU sources and the GPU adapter.
//!
//! Backends `declare` a channel's headline metric with its availability, then
//! `record` samples; the UI only reads. Sub-project 4 adds reductions and
//! consumers (probes, export, comparison) on top of these types.
use super::ChannelId;
use bevy::prelude::Resource;
use std::collections::HashMap;
mod trace;
pub use trace::Trace;

/// Stable metric identity and display text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MetricId {
    /// Machine name used by `specimens.toml`, e.g. `"population"`.
    pub name: &'static str,
    /// Short readout label, e.g. `"alive"`.
    pub label: &'static str,
    /// Unit shown in notes.
    pub unit: &'static str,
}

/// One measurement tied to the tick and reset epoch it describes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetricSample {
    /// Channel tick the value was measured at.
    pub tick: u64,
    /// Reset generation; traces never mix epochs.
    pub epoch: u64,
    /// Measured value.
    pub value: f64,
}

/// How (and whether) a metric is currently produced.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricAvailability {
    /// Sampled synchronously after every tick.
    Live,
    /// Sampled asynchronously at most once per period.
    Sampled {
        /// Seconds between requests.
        period_secs: f32,
    },
    /// Available but disabled until the viewer opts in.
    OptIn,
    /// Not producible; the reason is shown in the inspector.
    Unavailable(String),
}

/// Headline metric state for one channel.
#[derive(Clone, Debug)]
pub struct ChannelMetric {
    /// Which metric.
    pub id: MetricId,
    /// Current production mode.
    pub availability: MetricAvailability,
    /// Most recent accepted sample.
    pub latest: Option<MetricSample>,
    /// Recent history.
    pub trace: Trace,
}

/// Metric state keyed by channel; backends write, the UI reads.
#[derive(Resource, Debug, Default)]
pub struct Metrics(pub HashMap<ChannelId, ChannelMetric>);

impl Metrics {
    /// Create or update a channel's metric; history survives redeclaring the same id.
    pub fn declare(&mut self, channel: ChannelId, id: MetricId, availability: MetricAvailability) {
        let entry = self.0.entry(channel).or_insert_with(|| ChannelMetric {
            id,
            availability: availability.clone(),
            latest: None,
            trace: Trace::new(Trace::CAPACITY),
        });
        if entry.id != id {
            entry.id = id;
            entry.latest = None;
            entry.trace.clear();
        }
        entry.availability = availability;
    }

    /// Record a sample for a declared channel; returns whether the trace accepted it.
    pub fn record(&mut self, channel: ChannelId, sample: MetricSample) -> bool {
        let Some(metric) = self.0.get_mut(&channel) else {
            return false;
        };
        let accepted = metric.trace.push(sample);
        if accepted {
            metric.latest = Some(sample);
        }
        accepted
    }

    /// Metric state for `channel`.
    pub fn get(&self, channel: ChannelId) -> Option<&ChannelMetric> {
        self.0.get(&channel)
    }

    /// Keep only channels for which `open` holds.
    pub fn retain(&mut self, open: impl Fn(ChannelId) -> bool) {
        self.0.retain(|id, _| open(*id));
    }
}

/// Shape shared by CPU sources and the GPU adapter: describe, then sample when ready.
pub trait MetricSource {
    /// Metric identity and how it is produced.
    fn describe(&self) -> (MetricId, MetricAvailability);
    /// A sample if one is ready now.
    fn sample(&mut self) -> Option<MetricSample>;
}

/// Total Lenia mass: quantized densities summed and rescaled to 0..=1 per cell.
pub fn mass(densities: &[u8]) -> f64 {
    densities.iter().map(|d| f64::from(*d)).sum::<f64>() / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const POP: MetricId = MetricId {
        name: "population",
        label: "alive",
        unit: "cells",
    };

    #[test]
    fn record_requires_declaration_and_tracks_latest() {
        let mut metrics = Metrics::default();
        let id = ChannelId(3);
        let sample = |tick, value| MetricSample {
            tick,
            epoch: 0,
            value,
        };
        assert!(!metrics.record(id, sample(0, 1.0)));
        metrics.declare(id, POP, MetricAvailability::Live);
        assert!(metrics.record(id, sample(1, 4.0)));
        assert_eq!(metrics.get(id).unwrap().latest.unwrap().value, 4.0);
        metrics.declare(id, POP, MetricAvailability::Unavailable("lost".into()));
        assert_eq!(
            metrics.get(id).unwrap().trace.len(),
            1,
            "redeclaring keeps history"
        );
        metrics.retain(|open| open != id);
        assert!(metrics.get(id).is_none());
    }

    #[test]
    fn mass_is_density_sum_over_255() {
        assert_eq!(mass(&[255; 8]), 8.0);
        assert_eq!(mass(&[]), 0.0);
        assert!((mass(&[51, 51, 51, 51, 51]) - 1.0).abs() < 1e-12);
    }
}
