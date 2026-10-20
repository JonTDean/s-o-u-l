//! The single command vocabulary accepted by every channel backend.
use super::{ChannelId, Srgb};
use crate::automata::lenia::LeniaAction;
use bevy::prelude::Event;

/// A requested change to one channel; backends ignore actions that do not apply to them.
#[derive(Clone, Debug)]
pub enum ChannelAction {
    /// Make this channel the one shown on the stage.
    Select,
    /// Suspend (`true`) or resume (`false`) automatic ticks; halted channels stay paused.
    Pause(bool),
    /// Advance exactly one tick, only while paused.
    Step,
    /// Restore the initial condition, pause, and start a new metric epoch.
    Reset,
    /// Close the channel and release its id and resources.
    Remove,
    /// Change automatic stepping rate in steps per second (clamped to 1–60).
    SetRate(f32),
    /// Change the specimen colour used by painters and the voxel shader.
    SetColour(Srgb),
    /// Replace symbolic input; applied by the next `Reset`.
    SetInput(String),
    /// Enable or disable an opt-in metric source (Lenia GPU mass readback).
    SampleMetric(bool),
    /// Paint a paused cellular field; the first batch starts one undoable stroke.
    Paint {
        /// Cells crossed by this stroke segment.
        points: Vec<[i32; 2]>,
        /// Binary density or HPP direction bits.
        value: u8,
        /// Begin a new undoable stroke.
        start: bool,
    },
    /// Undo (`false`) or redo (`true`) a cellular stroke.
    UndoPaint(bool),
    /// Lenia-specific parameter and seed actions.
    Lenia(LeniaAction),
}

/// Channel command event, written by the lab UI and native qualification.
#[derive(Event, Clone, Debug)]
pub struct ChannelCommand {
    /// Target channel; unknown or removed ids are ignored by every backend.
    pub id: ChannelId,
    /// Requested change.
    pub action: ChannelAction,
}
