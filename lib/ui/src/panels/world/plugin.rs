//! Root plugin for all *in-game* (world) HUD overlays.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use engine::systems::state::AppState;

use crate::panels::world::minimap_overlay::MinimapSelection;

use super::{
    automata::AutomataPanelPlugin,
    pause_menu::PauseMenuPlugin,
    zoom_overlay::zoom_overlay,
    minimap_overlay::minimap_overlay,
};

pub struct WorldMenusPlugin;

impl Plugin for WorldMenusPlugin {
    fn build(&self, app: &mut App) {
        // one tiny helper resource shared by several panels
        app.init_resource::<MinimapSelection>();

        // sub-plugins
        app.add_plugins((AutomataPanelPlugin, PauseMenuPlugin))

            // free-floating HUD widgets – run only while playing
            .add_systems(
                EguiPrimaryContextPass,
                (zoom_overlay, minimap_overlay)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
