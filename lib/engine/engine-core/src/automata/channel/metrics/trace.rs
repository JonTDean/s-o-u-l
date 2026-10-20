//! Fixed-capacity per-channel metric history with epoch isolation.
use super::MetricSample;
use std::collections::VecDeque;

/// Ring buffer of one channel's headline metric, isolated per reset epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    samples: VecDeque<MetricSample>,
    capacity: usize,
    epoch: Option<u64>,
}

impl Trace {
    /// Samples kept per channel (the transport sparkline width).
    pub const CAPACITY: usize = 200;

    /// Empty trace holding at most `capacity` samples.
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
            epoch: None,
        }
    }

    /// Append; a newer epoch clears history, an older epoch or tick is rejected,
    /// and a repeated tick (e.g. while paused) replaces the last value.
    pub fn push(&mut self, sample: MetricSample) -> bool {
        match self.epoch {
            Some(epoch) if sample.epoch < epoch => return false,
            Some(epoch) if sample.epoch > epoch => self.samples.clear(),
            _ => {}
        }
        self.epoch = Some(sample.epoch);
        if let Some(last) = self.samples.back_mut() {
            if sample.tick < last.tick {
                return false;
            }
            if sample.tick == last.tick {
                *last = sample;
                return true;
            }
        }
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
        true
    }

    /// Forget all samples and the epoch.
    pub fn clear(&mut self) {
        self.samples.clear();
        self.epoch = None;
    }

    /// Samples oldest first.
    pub fn samples(&self) -> impl Iterator<Item = &MetricSample> {
        self.samples.iter()
    }

    /// Number of samples held.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether no samples are held.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Minimum and maximum value, for scaling the sparkline.
    pub fn range(&self) -> Option<(f64, f64)> {
        self.samples.iter().fold(None, |acc, s| match acc {
            None => Some((s.value, s.value)),
            Some((lo, hi)) => Some((lo.min(s.value), hi.max(s.value))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::channel::metrics::MetricSample;

    fn s(tick: u64, epoch: u64, value: f64) -> MetricSample {
        MetricSample { tick, epoch, value }
    }

    #[test]
    fn evicts_oldest_beyond_capacity() {
        let mut trace = Trace::new(3);
        for t in 0..5 {
            assert!(trace.push(s(t, 0, t as f64)));
        }
        let ticks: Vec<u64> = trace.samples().map(|x| x.tick).collect();
        assert_eq!(ticks, [2, 3, 4]);
    }

    #[test]
    fn newer_epoch_clears_and_older_epoch_is_rejected() {
        let mut trace = Trace::new(10);
        trace.push(s(5, 1, 1.0));
        assert!(!trace.push(s(9, 0, 9.0)), "stale epoch");
        assert!(trace.push(s(0, 2, 2.0)));
        assert_eq!(trace.len(), 1);
    }

    #[test]
    fn same_tick_replaces_and_older_tick_is_rejected() {
        let mut trace = Trace::new(10);
        trace.push(s(4, 0, 1.0));
        assert!(trace.push(s(4, 0, 2.0)));
        assert_eq!(trace.len(), 1);
        assert_eq!(trace.samples().last().unwrap().value, 2.0);
        assert!(!trace.push(s(3, 0, 0.0)));
    }

    #[test]
    fn range_spans_min_and_max() {
        let mut trace = Trace::new(10);
        assert_eq!(trace.range(), None);
        for (t, v) in [(0, 3.0), (1, -1.0), (2, 7.5)] {
            trace.push(s(t, 0, v));
        }
        assert_eq!(trace.range(), Some((-1.0, 7.5)));
    }
}
