//! Backend-neutral channel contract shared by the lab UI, qualification and both backends.
//!
//! Commands flow UI → `ChannelCommand` → backend routers (`ChannelSet::Route`); backends
//! tick (`Tick`), publish rows into [`Channels`] (`Publish`), and `Settle` repairs selection.
mod clock;
mod colour;
mod command;
pub mod metrics;

use crate::systems::schedule::MainSet;
use bevy::prelude::*;
pub use clock::StepClock;
pub use colour::{BACKGROUND, SIGNAL, SWATCHES, Srgb, ramp};
pub use command::{ChannelAction, ChannelCommand};

/// Stable handle for one open session; monotonically allocated, never reused, never shown.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelId(pub u32);

impl ChannelId {
    /// Placeholder for sessions built outside a workbench (tests, project files).
    pub const UNASSIGNED: Self = Self(u32::MAX);
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::UNASSIGNED
    }
}

/// Run-wide id allocator; also enforces the shared eight-channel capacity.
#[derive(Resource, Debug, Default)]
pub struct ChannelIds {
    next: u32,
    open: Vec<ChannelId>,
}

impl ChannelIds {
    /// Maximum simultaneously open channels across all backends.
    pub const CAPACITY: usize = 8;

    /// Allocate a fresh id, or `None` when eight channels are open.
    pub fn allocate(&mut self) -> Option<ChannelId> {
        if self.open.len() >= Self::CAPACITY {
            return None;
        }
        let id = ChannelId(self.next);
        self.next += 1;
        self.open.push(id);
        Some(id)
    }

    /// Return capacity; ids that are not open are ignored.
    pub fn release(&mut self, id: ChannelId) {
        self.open.retain(|open| *open != id);
    }

    /// Number of currently open channels.
    pub fn open(&self) -> usize {
        self.open.len()
    }
}

/// Stepping rates offered by the transport, in steps per second.
pub const RATE_PRESETS: [f32; 4] = [1.0, 10.0, 30.0, 60.0];

/// Where a channel executes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// Inspectable CPU session in `models::workbench`.
    Cpu,
    /// GPU Lenia owner entity.
    Gpu,
}

/// Lifecycle state shown by tab lamps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Advancing automatically.
    Running,
    /// Waiting for resume or single steps.
    Paused,
    /// Terminal or resource-bounded; only Reset continues.
    Halted,
}

/// Per-channel pacing and colour, owned by the backend.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct ChannelSettings {
    /// Automatic stepping rate in steps per second.
    pub rate_hz: f32,
    /// Specimen colour.
    pub colour: Srgb,
}

impl ChannelSettings {
    /// Settings at `rate_hz` in the default amber.
    pub fn new(rate_hz: f32) -> Self {
        Self {
            rate_hz,
            colour: SIGNAL,
        }
    }
}

/// One read-only row published by a backend each frame.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelRow {
    /// Stable handle.
    pub id: ChannelId,
    /// Catalog id, e.g. `"life:conway"` or `"lenia"`.
    pub kind: String,
    /// Executing backend.
    pub backend: Backend,
    /// Lamp state.
    pub status: Status,
    /// Ticks since the latest reset (requested ticks for Lenia).
    pub tick: u64,
    /// Reset generation; metric samples from older epochs are discarded.
    pub epoch: u64,
    /// Pacing and colour.
    pub settings: ChannelSettings,
}

/// Snapshot of every open channel plus the stage selection.
#[derive(Resource, Debug, Default)]
pub struct Channels {
    /// Rows in creation (id) order.
    pub rows: Vec<ChannelRow>,
    /// Channel shown on the stage; repaired by `ChannelSet::Settle`.
    pub selected: Option<ChannelId>,
}

impl Channels {
    /// Replace every row of `backend`, keeping creation order across backends.
    pub fn publish(&mut self, backend: Backend, rows: impl IntoIterator<Item = ChannelRow>) {
        self.rows.retain(|row| row.backend != backend);
        self.rows.extend(rows);
        self.rows.sort_by_key(|row| row.id);
    }

    /// Row for `id`, if open.
    pub fn get(&self, id: ChannelId) -> Option<&ChannelRow> {
        self.rows.iter().find(|row| row.id == id)
    }

    /// Position of `id` in tab order.
    pub fn index_of(&self, id: ChannelId) -> Option<usize> {
        self.rows.iter().position(|row| row.id == id)
    }

    /// Row shown on the stage.
    pub fn selected_row(&self) -> Option<&ChannelRow> {
        self.selected.and_then(|id| self.get(id))
    }

    /// Whether the shared capacity is exhausted.
    pub fn is_full(&self) -> bool {
        self.rows.len() >= ChannelIds::CAPACITY
    }
}

/// Per-frame ordering: route commands → tick → publish rows → settle selection.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChannelSet {
    /// Apply `ChannelCommand`s and spawn/clear requests.
    Route,
    /// Advance clocks and simulations.
    Tick,
    /// Publish rows and metric samples.
    Publish,
    /// Repair selection and drop state of closed channels.
    Settle,
}

/// Registers the contract; every backend adds it through [`ensure_plugin`].
pub struct ChannelPlugin;

impl Plugin for ChannelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChannelCommand>()
            .init_resource::<Channels>()
            .init_resource::<ChannelIds>()
            .init_resource::<metrics::Metrics>()
            .configure_sets(
                Update,
                (
                    ChannelSet::Route,
                    ChannelSet::Tick,
                    ChannelSet::Publish,
                    ChannelSet::Settle,
                )
                    .chain()
                    .after(MainSet::Input),
            )
            .add_systems(
                Update,
                (
                    select.in_set(ChannelSet::Route),
                    settle.in_set(ChannelSet::Settle),
                ),
            );
    }
}

/// Add [`ChannelPlugin`] unless another backend already did.
pub fn ensure_plugin(app: &mut App) {
    if !app.is_plugin_added::<ChannelPlugin>() {
        app.add_plugins(ChannelPlugin);
    }
}

fn select(mut commands: EventReader<ChannelCommand>, mut channels: ResMut<Channels>) {
    for command in commands.read() {
        if matches!(command.action, ChannelAction::Select) && channels.get(command.id).is_some() {
            channels.selected = Some(command.id);
        }
    }
}

/// After removal, select the closest earlier channel, else the first; never a closed id.
fn settle(mut channels: ResMut<Channels>, mut metrics: ResMut<metrics::Metrics>) {
    let missing = channels.selected.filter(|id| channels.get(*id).is_none());
    if let Some(gone) = missing {
        let previous = channels.rows.iter().rev().find(|row| row.id < gone);
        channels.selected = previous.or(channels.rows.first()).map(|row| row.id);
    } else if channels.selected.is_none() {
        channels.selected = channels.rows.first().map(|row| row.id);
    }
    let open: Vec<ChannelId> = channels.rows.iter().map(|row| row.id).collect();
    metrics.retain(|id| open.contains(&id));
}

#[cfg(test)]
mod tests;
