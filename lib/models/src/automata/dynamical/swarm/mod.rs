pub mod ant_colony;
pub mod boids;

/// Registers both bounded agent examples with the shared workbench.
pub struct SwarmPlugin;
impl bevy::prelude::Plugin for SwarmPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use crate::workbench::catalog::{Kind, register};
        register(app, &[Kind::Boids, Kind::Ants]);
    }
}
