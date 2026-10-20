//! Opt-in GPU metric adapter: throttled `LeniaProbe` readbacks reduced to total mass.
//!
//! Ordinary rendering performs no readbacks; sampling starts only after a
//! `SampleMetric(true)` command and pauses while [`ReadbackBusy`] is set.
use bevy::prelude::*;
use engine_core::automata::{
    channel::{
        ChannelAction, ChannelCommand, ChannelId,
        metrics::{MetricAvailability, MetricId, MetricSample, Metrics, mass},
    },
    lenia::{LeniaSimulation, ReadbackBusy},
};
use std::collections::HashMap;

/// Lenia mass: Σ density / 255.
pub const MASS: MetricId = MetricId {
    name: "mass",
    label: "mass",
    unit: "density",
};
/// Minimum seconds between readback requests (4 Hz).
pub const PERIOD_SECS: f32 = 0.25;
/// Seconds after which an unanswered request is reported as unavailable.
pub const TIMEOUT_SECS: f32 = 2.0;

/// Finished `(epoch, tick, value)` or the GPU error.
pub type Snapshot = Result<(u64, u64, f64), String>;

/// Readback mailbox seam; implemented by `engine_gpu::LeniaProbe` and by test fakes.
pub trait SnapshotMailbox {
    /// Ask for one asynchronous readback of `owner`.
    fn request(&self, owner: Entity);
    /// A finished readback for `owner`, if any.
    fn take(&self, owner: Entity) -> Option<Snapshot>;
}

#[cfg(feature = "gpu-compute")]
impl SnapshotMailbox for engine_gpu::LeniaProbe {
    fn request(&self, owner: Entity) {
        engine_gpu::LeniaProbe::request(self, owner);
    }
    fn take(&self, owner: Entity) -> Option<Snapshot> {
        engine_gpu::LeniaProbe::take(self, owner).map(|r| {
            r.map(|s| {
                (
                    s.epoch,
                    s.tick,
                    s.continuous.as_ref().map_or_else(
                        || mass(&s.densities),
                        |c| c.values.iter().map(|v| *v as f64).sum(),
                    ),
                )
            })
        })
    }
}

#[derive(Clone, Debug, Default)]
struct Cycle {
    elapsed: f32,
    waiting: bool,
    error: Option<String>,
}

/// Channels that opted in, and where each is in its request cycle.
#[derive(Resource, Default)]
pub struct MassSampling(HashMap<ChannelId, Cycle>);

/// Apply `SampleMetric` toggles.
pub fn toggle(mut commands: EventReader<ChannelCommand>, mut sampling: ResMut<MassSampling>) {
    for command in commands.read() {
        match command.action {
            ChannelAction::SampleMetric(true) => {
                sampling.0.entry(command.id).or_insert(Cycle {
                    elapsed: PERIOD_SECS,
                    ..default()
                });
            }
            ChannelAction::SampleMetric(false) => {
                sampling.0.remove(&command.id);
            }
            _ => {}
        }
    }
}

/// Request, collect, reduce and publish mass for opted-in owners; declare the rest `OptIn`.
pub fn sample<M: SnapshotMailbox + Resource>(
    time: Res<Time>,
    probe: Res<M>,
    busy: Option<Res<ReadbackBusy>>,
    owners: Query<(Entity, &ChannelId, &LeniaSimulation)>,
    mut sampling: ResMut<MassSampling>,
    mut metrics: ResMut<Metrics>,
) {
    let busy = busy.is_some_and(|b| b.0);
    let dt = time.delta_secs();
    for (entity, id, sim) in &owners {
        let Some(cycle) = sampling.0.get_mut(id) else {
            metrics.declare(*id, MASS, MetricAvailability::OptIn);
            continue;
        };
        if !busy {
            cycle.elapsed += dt;
            advance(cycle, &*probe, entity, sim, *id, &mut metrics);
        }
        let availability = match &cycle.error {
            Some(error) => MetricAvailability::Unavailable(error.clone()),
            None => MetricAvailability::Sampled {
                period_secs: PERIOD_SECS,
            },
        };
        metrics.declare(*id, MASS, availability);
    }
    sampling
        .0
        .retain(|id, _| owners.iter().any(|(_, owner, _)| owner == id));
}

fn advance(
    cycle: &mut Cycle,
    probe: &impl SnapshotMailbox,
    entity: Entity,
    sim: &LeniaSimulation,
    id: ChannelId,
    metrics: &mut Metrics,
) {
    if !cycle.waiting {
        if cycle.elapsed >= PERIOD_SECS {
            probe.request(entity);
            (cycle.waiting, cycle.elapsed) = (true, 0.0);
        }
        return;
    }
    match probe.take(entity) {
        Some(Ok((epoch, tick, value))) => {
            (cycle.waiting, cycle.error) = (false, None);
            // Snapshots from before the latest reset belong to a finished trace.
            if epoch >= sim.epoch {
                let available = MetricAvailability::Sampled {
                    period_secs: PERIOD_SECS,
                };
                metrics.declare(id, MASS, available);
                metrics.record(id, MetricSample { tick, epoch, value });
            }
        }
        Some(Err(error)) => (cycle.waiting, cycle.error) = (false, Some(error)),
        None if cycle.elapsed >= TIMEOUT_SECS => {
            (cycle.waiting, cycle.error) = (false, Some("GPU readback timed out".into()));
        }
        None => {}
    }
}
