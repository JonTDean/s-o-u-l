//! Convenience re-exports for engine-core consumers.

pub use crate::{
    automata::AutomatonInfo,
    events::*,
    systems::{registry::*, schedule::*, state::AppState},
    world::*,
};

pub use bevy::prelude::*;
