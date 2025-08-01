//! top-level of **engine-render** – nothing here should leak internals.

mod render;

use bevy_ecs::resource::Resource;

/* public surface ---------------------------------------------------- */
pub use render::{
    CameraPlugin,
    WorldCamera,
    WorldGrid,
    AutomataMaterial,
    AutomataParams,
};

/* helper wrapper ---------------------------------------------------- */
#[derive(Resource, Clone)]
pub struct RuleParams(pub serde_json::Value);

/* convenience re-exports ------------------------------------------- */
pub mod prelude;
pub mod command_executor;
pub mod plugin;
pub mod debug_plugin;
