pub mod dense;
pub mod sparse;
pub mod plugin;

use bevy::prelude::Resource;

/// Convenience wrapper so the app can do
/// `.add_plugins(StepperPlugin::<MyRule>{ … })`
#[derive(Resource, Clone)]
pub struct RuleParams(pub serde_json::Value);

