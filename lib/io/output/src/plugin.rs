//! Output layer root‑plugin – bundles UI, GPU rendering and HUD overlays.

use bevy::prelude::*;

use crate::{
    rendering::{active::plugin::ActiveAutomataRenderPlugin, grid2d::Grid2DRenderPlugin}, ui::panels::{
        main_menu::MainMenuPlugin,
        world::WorldMenusPlugin,          // ← new world‑HUD plugin
    }
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
            // 2‑D cell‑grid renderer (wgpu)
            // Grid2DRenderPlugin,
            // GPU renderer for active automata
            ActiveAutomataRenderPlugin,
        ));
    }
}
