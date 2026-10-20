//! Headline metric per CPU family, sampled synchronously after each tick.
use crate::{
    automata::dynamical::swarm::boids::Boid,
    workbench::{
        catalog::Kind,
        plugin::Workbench,
        session::{Model, Session},
        symbolic::machine::Symbolic,
    },
};
use bevy::prelude::*;
use engine_core::automata::channel::metrics::{
    MetricAvailability, MetricId, MetricSample, MetricSource, Metrics,
};

const fn metric(name: &'static str, label: &'static str, unit: &'static str) -> MetricId {
    MetricId { name, label, unit }
}

/// Live cells in the newest elementary row.
pub const LIVE_CELLS: MetricId = metric("live_cells", "alive", "cells");
/// Live cells of a 2-D grid.
pub const POPULATION: MetricId = metric("population", "alive", "cells");
/// HPP particles (one per set direction bit); conserved by collisions.
pub const PARTICLES: MetricId = metric("particles", "particles", "particles");
/// Machine transitions performed.
pub const STEPS: MetricId = metric("steps", "steps", "transitions");
/// L-system word length.
pub const LENGTH: MetricId = metric("length", "symbols", "symbols");
/// Boid alignment: |mean unit heading|, 0 (disordered) to 1 (aligned).
pub const POLARISATION: MetricId = metric("polarisation", "order", "0–1");
/// Ant food deliveries home.
pub const FOOD: MetricId = metric("food", "food", "deliveries");

/// Headline metric for each family.
pub fn metric_for(kind: Kind) -> MetricId {
    match kind {
        Kind::Rule30 | Kind::Rule110 => LIVE_CELLS,
        Kind::Life => POPULATION,
        Kind::Hpp => PARTICLES,
        Kind::Koch => LENGTH,
        Kind::Boids => POLARISATION,
        Kind::Ants => FOOD,
        _ => STEPS,
    }
}

/// Current metric value from model state.
pub fn value(session: &Session) -> f64 {
    match &session.model {
        Model::Grid(grid) => match session.kind {
            Kind::Rule30 | Kind::Rule110 => {
                grid.history
                    .back()
                    .map_or(0, |row| row.iter().filter(|v| **v != 0).count()) as f64
            }
            Kind::Hpp => grid.bytes().iter().map(|v| v.count_ones()).sum::<u32>() as f64,
            _ => grid.bytes().iter().filter(|v| **v != 0).count() as f64,
        },
        Model::Symbolic(Symbolic::Koch { word, .. }) => word.chars().count() as f64,
        Model::Symbolic(_) => session.ticks as f64,
        Model::Boids(flock) => polarisation(&flock.boids),
        Model::Ants(arena) => arena.deliveries as f64,
    }
}

fn polarisation(boids: &[Boid]) -> f64 {
    if boids.is_empty() {
        return 0.0;
    }
    let sum: glam::Vec2 = boids.iter().map(|b| b.velocity.normalize_or_zero()).sum();
    f64::from(sum.length() / boids.len() as f32)
}

/// One session viewed as a live metric source.
pub struct SessionMetric<'a>(pub &'a Session);

impl MetricSource for SessionMetric<'_> {
    fn describe(&self) -> (MetricId, MetricAvailability) {
        (metric_for(self.0.kind), MetricAvailability::Live)
    }
    fn sample(&mut self) -> Option<MetricSample> {
        Some(MetricSample {
            tick: self.0.ticks,
            epoch: self.0.epoch,
            value: value(self.0),
        })
    }
}

/// Declare and sample every CPU session's headline metric.
pub(in crate::workbench) fn publish(bench: Res<Workbench>, mut metrics: ResMut<Metrics>) {
    for session in &bench.sessions {
        let mut source = SessionMetric(session);
        let (id, availability) = source.describe();
        metrics.declare(session.channel, id, availability);
        if let Some(sample) = source.sample() {
            metrics.record(session.channel, sample);
        }
    }
}
