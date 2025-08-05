//! top-level of **engine-render** – nothing here should leak internals.

#![warn(missing_docs)]

pub mod render;

use bevy_ecs::resource::Resource;

/* helper wrapper ---------------------------------------------------- */

#[derive(Resource, Clone)]
/// Wrapper around rule parameters passed to rendering materials.
pub struct RuleParams(pub serde_json::Value);

/* convenience re-exports ------------------------------------------- */
pub mod command_executor;
pub mod plugin;
pub mod prelude;
