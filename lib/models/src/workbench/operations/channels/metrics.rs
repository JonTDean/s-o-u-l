//! Headline metric per CPU family (completed in Task 5).
use crate::workbench::plugin::Workbench;
use bevy::prelude::*;

/// Publish CPU metric samples.
pub(in crate::workbench) fn publish(_bench: Res<Workbench>) {}
