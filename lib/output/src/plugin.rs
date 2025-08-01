//! Output layer root‑plugin – bundles UI, GPU rendering and HUD overlays.

use bevy::prelude::*;

use crate::{
    rendering::{
        active::plugin::ActiveAutomataRenderPlugin,
    },
    ui::panels::{main_menu::MainMenuPlugin, world::WorldMenusPlugin},
};

/// Adds every “output” feature: menus, HUD, file‑export, renderers…
pub struct OutputPlugin;

impl Plugin for OutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // egui‑based menu stack (main menu, scenario screens…)
            MainMenuPlugin,
            // in‑scene HUD overlays (Active Automata, …)
            WorldMenusPlugin,
            // Active‑cell mask renderer
            ActiveAutomataRenderPlugin,
        ));
    }
}
