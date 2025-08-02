//! Output-layer root plug-in – bundles menus, HUD overlays and the CPU
//! active-grid renderer.

use bevy::prelude::*;
use crate::{
    panels::{
        main_menu::MainMenuPlugin,
        world::WorldMenusPlugin,
    },
    styles::fade::FadePlugin,
};

/// Adds every “output” feature: menus, HUD, file-export, renderers …
pub struct OutputPlugin;


/*
    Layer Diagram
        ┌────────────────────────────┐
        │   ui::OutputPlugin         │   ← egui menus, HUD, fade, etc.
        │   ├─ ui::panels::MainMenu  │
        │   ├─ ui::panels::WorldHUD  │
        │   └─ engine_render::ActiveAutomataRenderPlugin (re-export) ─┐
        └────────────────────────────┘                               │
                                                                    │ public API
        ┌─────────────────────────────────────────────────────────────▼────────────┐
        │           engine_render::EngineRenderPlugin                              │
        │   ├─ render::camera::CameraPlugin      (world + UI cameras)              │
        │   └─ render::voxel_grid::VoxelGridPlugin  (GPU atlas helper)              │
        └──────────────────────────────────────────────────────────────────────────┘
*/
impl Plugin for OutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
               /* egui-based menu stack (main-menu, editors …)            */
               MainMenuPlugin,
               /* in-scene HUD overlays (automata list, minimap, pause …) */
               WorldMenusPlugin,
               /* global fade-in / fade-out transitions                   */
               FadePlugin,
           ));
    }
}
