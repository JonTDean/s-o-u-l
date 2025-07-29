//! Output layer root-plugin – bundles UI, GPU rendering and HUD overlays.

use bevy::prelude::*;

use crate::{
    rendering::active::plugin::ActiveAutomataRenderPlugin,
    ui::{fade::FadePlugin, panels::{main_menu::MainMenuPlugin, world::WorldMenusPlugin}},
};

/// Adds every “output” feature: menus, HUD, file-export, renderers…
pub struct OutputPlugin;

impl Plugin for OutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // egui-based menu stack (main-menu, scenario editors…)
            MainMenuPlugin,
            // in-scene HUD overlays (Active-automata list, minimap, pause-menu…)
            WorldMenusPlugin,
            // CPU active-cell-mask renderer (quad + texture per automaton)
            ActiveAutomataRenderPlugin,
            /* global fade‑in / fade‑out transitions                         */
            FadePlugin, 
        ));
    }
}
