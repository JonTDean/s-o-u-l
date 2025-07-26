pub mod dense;
pub mod sparse;
pub mod plugin;
pub mod dense_parallel;

use bevy::prelude::Resource;

/// Convenience wrapper so the app can do
/// `.add_plugins(StepperPlugin::<MyRule>{ … })`
#[derive(Resource, Clone)]
pub struct RuleParams(pub serde_json::Value);

