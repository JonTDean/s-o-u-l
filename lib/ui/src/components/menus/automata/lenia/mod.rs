//! Controls for the qualified Lenia evolution path.
use bevy::prelude::*;
mod panel;
mod parameters;

pub(super) struct LeniaWorkbenchPlugin;

impl Plugin for LeniaWorkbenchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            panel::show.run_if(in_state(engine_core::prelude::AppState::InGame)),
        );
    }
}
