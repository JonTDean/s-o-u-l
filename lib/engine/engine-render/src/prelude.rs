//! Convenient glob import for downstream game code.

pub use bevy::prelude::*;

pub use crate::render::{
    CameraPlugin,
    ZoomInfo,
    AutomataRenderMap,
    WorldCamera,
    WorldGrid,
    AutomataMaterial,
    AutomataParams,
};

pub use crate::plugin;