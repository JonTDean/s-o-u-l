//! Rendering infrastructure for S.O.U.L.
//!
//! `engine-render` bridges the simulation data to Bevy's renderer. It
//! provides camera management, CPU and optional GPU based grid rendering as
//! well as a small command executor for spawning or clearing automata.
//! Downstream crates interact with this layer solely through its public
//! plugins and types – internal modules remain private.

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
pub use render::camera::systems::ZoomInfo;
pub use render::active::plugin::ActiveAutomataRenderPlugin;

/* helper wrapper ---------------------------------------------------- */
#[derive(Resource, Clone)]
pub struct RuleParams(pub serde_json::Value);

/* convenience re-exports ------------------------------------------- */
pub mod prelude;
pub mod command_executor;
pub mod plugin;