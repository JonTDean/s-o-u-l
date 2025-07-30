use bevy::ecs::resource::Resource;

pub mod stepper;
pub mod render_bridge;
pub mod plugin;
pub mod gpu;
pub mod worldgrid;
pub mod components;
pub mod camera_manager;
pub mod material;
pub mod active;
pub mod floating_origin;

/// Convenience wrapper so the app can do
/// `.add_plugins(StepperPlugin::<MyRule>{ … })`
#[derive(Resource, Clone)]
pub struct RuleParams(pub serde_json::Value);

